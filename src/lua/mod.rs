mod event_handler;
mod signal;

use crate::{consts, server};
use anyhow::{Context, Result};
use mlua::{Lua, LuaSerdeExt, RegistryKey};
use moka::future::Cache;
use sea_orm::DatabaseConnection;
use serenity::all::{ChannelId, GuildId, Http};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex as StdMutex};

use crate::Data;
use crate::consts::DATA_DIR;
use crate::db::get_or_create_server_table_cached;
use crate::lua::signal::create_signal;

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Default)]
pub struct BotCallbacks {
    ready_events: HashMap<u64, RegistryKey>,
    message_create_events: HashMap<u64, RegistryKey>,
}

#[derive(Copy, Clone)]
enum EventType {
    Ready,
    MessageCreate,
}

pub use event_handler::lua_event_handler;

pub fn load_scripts(lua: &Lua, directory: impl AsRef<Path>) -> Result<()> {
    let script_dir = directory.as_ref();

    if !script_dir.exists() || !script_dir.is_dir() {
        tracing::warn!(
            "directory '{:?}' not found. skipping script loading",
            script_dir
        );
        return Ok(());
    }

    for entry in fs::read_dir(script_dir).context("failed to read scripts directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file()
            && path
                .extension()
                .is_some_and(|ext| ext == "luau" || ext == "lua")
        {
            tracing::info!(
                "Compiling and loading script: {:?}",
                path.file_name().unwrap_or_default()
            );

            let code = fs::read_to_string(&path)
                .with_context(|| format!("failed to read lua script file {}", path.display()))?;

            let file_str = path.to_str().unwrap_or("unknown_file");

            let chunk_name = format!("={file_str}");

            if let Err(e) = lua.load(&code).set_name(&chunk_name).exec() {
                tracing::error!("failed to parse or execute {:?}:\n{}", path, e);
            }
        }
    }

    Ok(())
}

// i dont care for now
#[allow(clippy::too_many_lines)]
pub fn configure_lua_env(
    lua: &Lua,
    callbacks: &Arc<StdMutex<BotCallbacks>>,
    http: &Arc<Http>,
    server_cache: &Cache<u64, server::Model>,
    db: &DatabaseConnection,
    modules_path: &PathBuf,
) -> anyhow::Result<()> {
    let globals = lua.globals();

    // ==========================================
    // internal registries
    // ==========================================
    let registry = lua.create_table()?;
    let loaded = lua.create_table()?;

    // ==========================================
    // @skekbot/log & override print
    // ==========================================
    let log_backend =
        lua.create_function(|_, (level, location, message): (String, String, String)| {
            match level.to_uppercase().as_str() {
                "ERROR" => tracing::error!("({}) {}", location, message),
                "WARN" => tracing::warn!("({}) {}", location, message),
                _ => tracing::info!("({}) {}", location, message),
            }
            Ok(())
        })?;

    let log_module: mlua::Table = lua
        .load(
            r##"
        local log_backend = ...
        local function get_caller_info(stack_level)
            local source, line = debug.info(stack_level, "sl")
            if not source then return "unknown" end
            
            -- Strip [string "name"] wrapper
            local clean_name = string.match(source, '^%[string "(.-)"%]$')
            if clean_name then
                source = clean_name
            end
            
            if string.sub(source, 1, 1) == "=" or string.sub(source, 1, 1) == "@" then
                source = string.sub(source, 2)
            end
            return source .. ":" .. tostring(line)
        end

        print = function(...)
            local num_args = select("#", ...)
            local str = {}
            for i = 1, num_args do
                table.insert(str, tostring(select(i, ...)))
            end
            log_backend("INFO", get_caller_info(3), table.concat(str, "\t"))
        end

        return {
            log = function(level, message)
                log_backend(level, get_caller_info(3), tostring(message))
            end
        }
        "##,
        )
        .call(log_backend)?;

    registry.set("@skekbot/log", log_module)?;

    // ==========================================
    // @skekbot/utils
    // ==========================================
    let utils_table = lua.create_table()?;

    let start_time = std::time::Instant::now();
    let uptime_helper = lua.create_function(move |_, ()| Ok(start_time.elapsed().as_secs()))?;
    utils_table.set("getUptime", uptime_helper)?;

    let sleep_helper = lua.create_async_function(|_, seconds: f64| async move {
        tokio::time::sleep(std::time::Duration::from_secs_f64(seconds)).await;
        Ok(())
    })?;
    utils_table.set("sleep", sleep_helper)?;

    registry.set("@skekbot/utils", utils_table)?;

    // ==========================================
    // @skekbot/db
    // ==========================================
    let db_table = lua.create_table()?;
    let db_clone = db.clone();
    let cache_clone = server_cache.clone();

    let get_settings = lua.create_async_function(move |lua, server_id: String| {
        let db = db_clone.clone();
        let cache = cache_clone.clone();

        async move {
            let guild_id_u64 = server_id.parse::<u64>().map_err(|_| {
                mlua::Error::RuntimeError("Invalid Server ID: Must be a numeric string".to_string())
            })?;

            let guild_id = GuildId::new(guild_id_u64);

            let model = get_or_create_server_table_cached(&guild_id, &db, &cache)
                .await
                .map_err(|e| mlua::Error::RuntimeError(format!("Database Error: {e}")))?;

            let lua_value = lua.to_value(&model)?;
            Ok(lua_value)
        }
    })?;

    db_table.set("getOrCreateServerSettings", get_settings)?;
    registry.set("@skekbot/db", db_table)?;

    // ==========================================
    // @skekbot/events
    // ==========================================
    let events_table = lua.create_table()?;
    events_table.set(
        "OnReady",
        create_signal(lua, &Arc::clone(callbacks), EventType::Ready)?,
    )?;
    events_table.set(
        "OnMessageCreate",
        create_signal(lua, &Arc::clone(callbacks), EventType::MessageCreate)?,
    )?;

    registry.set("@skekbot/events", events_table)?;

    // ==========================================
    // @skekbot/rest
    // ==========================================
    let rest_table = lua.create_table()?;
    let http_clone = Arc::clone(http);

    let send_message =
        lua.create_async_function(move |_, (channel_id_str, content): (String, String)| {
            let http = Arc::clone(&http_clone);
            async move {
                let channel_id_u64 = channel_id_str.parse::<u64>().map_err(|_| {
                    mlua::Error::RuntimeError(
                        "invalid channel id: must be a numeric string".to_string(),
                    )
                })?;

                let channel_id = ChannelId::new(channel_id_u64);
                let message_builder = serenity::builder::CreateMessage::new().content(content);

                channel_id
                    .send_message(&http, message_builder)
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(format!("discord api error: {e}")))?;
                Ok(())
            }
        })?;

    rest_table.set("sendMessage", send_message)?;
    registry.set("@skekbot/rest", rest_table)?;

    // store in named registry to prevent sandboxing from wiping them
    lua.set_named_registry_value("__SKEKBOT_REGISTRY", registry)?;
    lua.set_named_registry_value("__SKEKBOT_LOADED", loaded)?;

    // ==========================================
    // path resolution & sandboxed require
    // ==========================================
    let base_dir = modules_path
        .canonicalize()
        .unwrap_or_else(|_| modules_path.clone());

    let require_fn = lua.create_function(move |lua, path: String| {
        // virtual modules (@skekbot/*)
        if path.starts_with("@skekbot/") {
            let registry: mlua::Table = lua.named_registry_value("__SKEKBOT_REGISTRY")?;
            let module: mlua::Value = registry.get(path.as_str())?;
            if !module.is_nil() {
                return Ok(module);
            }
            return Err(mlua::Error::RuntimeError(format!(
                "virtual module '{path}' not found"
            )));
        }

        // cache check
        let loaded: mlua::Table = lua.named_registry_value("__SKEKBOT_LOADED")?;
        let cached: mlua::Value = loaded.get(path.as_str())?;
        if !cached.is_nil() {
            return Ok(cached);
        }

        // lua module path resolution (handle both dots and direct paths)
        let is_direct_path = path.contains('/')
            || path.contains('\\')
            || path.ends_with(".luau")
            || path.ends_with(".lua");

        let target_path = if is_direct_path {
            base_dir.join(&path)
        } else {
            // standard lua dot notation: "a.b.c" -> "a/b/c"
            base_dir.join(path.replace('.', "/"))
        };

        // search for valid extensions if none were provided
        let candidates = if is_direct_path {
            vec![target_path]
        } else {
            vec![
                target_path.with_extension("luau"),
                target_path.with_extension("lua"),
                target_path.join("init.luau"),
                target_path.join("init.lua"),
            ]
        };

        let mut found_path = None;
        for candidate in candidates {
            if candidate.is_file() {
                found_path = Some(candidate);
                break;
            }
        }

        let resolved_target = match found_path {
            Some(p) => p,
            None => {
                return Err(mlua::Error::RuntimeError(format!(
                    "module '{path}' not found"
                )));
            }
        };

        // file sandboxing
        // canonicalise evaluates symlinks, `..`, and `.`. if the file doesn't exist, it errors here
        let resolved_path = resolved_target.canonicalize().map_err(|e| {
            mlua::Error::RuntimeError(format!("cannot resolve module '{path}': {e}"))
        })?;

        // check if the completely resolved physical path starts with our base_dir
        if !resolved_path.starts_with(&base_dir) {
            return Err(mlua::Error::RuntimeError(format!(
                "access denied: module '{path}' is outside the allowed modules directory"
            )));
        }

        // execution
        let file_content = std::fs::read_to_string(&resolved_path)
            .map_err(|e| mlua::Error::RuntimeError(format!("cannot open file '{path}': {e}")))?;

        // prepend "@" to the path string here!
        let mut result: mlua::Value = lua
            .load(&file_content)
            .set_name(format!("@{path}"))
            .call(())?;

        if result.is_nil() {
            result = mlua::Value::Boolean(true);
        }

        loaded.set(path.as_str(), result.clone())?;

        Ok(result)
    })?;

    globals.set("require", require_fn)?;

    Ok(())
}

pub async fn reload_scripts(data: &Data, http: Arc<serenity::all::Http>) -> anyhow::Result<()> {
    tracing::info!("reloading luau scripts...");

    {
        let mut callbacks = data
            .lua_callbacks
            .lock()
            .map_err(|_| anyhow::anyhow!("lua callbacks mutex is poisoned"))?;

        callbacks.ready_events.clear();
        callbacks.message_create_events.clear();
    } // lock drops here

    let lua = data.lua.lock().await;

    configure_lua_env(
        &lua,
        &Arc::clone(&data.lua_callbacks),
        &http,
        &data.server_cache,
        &data.db,
        &Path::new(DATA_DIR).join("luau").join("modules"),
    )?;

    let scripts_path = std::path::Path::new(DATA_DIR).join("luau").join("scripts");
    load_scripts(&lua, scripts_path)?;

    tracing::info!("luau reload complete!");

    Ok(())
}
