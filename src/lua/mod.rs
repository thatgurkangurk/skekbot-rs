mod builder;
mod event_handler;
pub mod modules;
mod signal;

use crate::server;
use anyhow::{Context, Result};
use mlua::{Lua, RegistryKey};
use moka::future::Cache;
use sea_orm::DatabaseConnection;
use serenity::all::Http;
use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex as StdMutex};
use std::{env, fs};

use crate::Data;
use crate::consts::DATA_DIR;

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
    db_conn: &DatabaseConnection,
    modules_path: &Path,
) -> anyhow::Result<()> {
    let globals = lua.globals();

    // ==========================================
    // internal registries
    // ==========================================
    let registry = lua.create_table()?;
    let loaded = lua.create_table()?;

    // ==========================================
    // register virtual modules
    // ==========================================
    let builders = vec![
        modules::utils::setup(lua)?,
        modules::rest::setup(lua, http)?,
        modules::log::setup(lua)?,
        modules::db::setup(lua, db_conn, server_cache)?,
        modules::events::setup(lua, callbacks)?,
    ];

    for builder in &builders {
        builder.register(&registry)?;
    }

    let types_dir = std::path::PathBuf::from("./types/skekbot");
    if !types_dir.exists() {
        std::fs::create_dir_all(&types_dir)?;
    }

    let is_generate_cmd = env::args().nth(1).as_deref() == Some("generate-types");
    let is_debug_mode = cfg!(debug_assertions);

    if is_generate_cmd || is_debug_mode {
        let types_dir = std::path::PathBuf::from("./types/skekbot");
        if !types_dir.exists() {
            std::fs::create_dir_all(&types_dir)?;
        }

        for builder in &builders {
            builder.emit_type_file(&types_dir)?;
        }

        if is_generate_cmd {
            tracing::info!("type generation complete. exiting...");
        }
    }

    // store in named registry to prevent sandboxing from wiping them
    lua.set_named_registry_value("__SKEKBOT_REGISTRY", registry)?;
    lua.set_named_registry_value("__SKEKBOT_LOADED", loaded)?;

    // ==========================================
    // path resolution & sandboxed require
    // ==========================================
    let base_dir = modules_path
        .canonicalize()
        .unwrap_or_else(|_| modules_path.to_path_buf());

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
            || std::path::Path::new(&path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("luau"))
            || std::path::Path::new(&path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("lua"));

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

        let Some(resolved_target) = found_path else {
            return Err(mlua::Error::RuntimeError(format!(
                "module '{path}' not found"
            )));
        };

        // file sandboxing
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

        // prepend @ to the path so Luau formats errors as proper files, not [string]
        let chunk_name = format!("@{path}");
        let result: mlua::Value = lua.load(&file_content).set_name(chunk_name).call(())?;

        if result.is_nil() {
            return Err(mlua::Error::RuntimeError(format!(
                "the module at '{path}' is empty"
            )));
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
