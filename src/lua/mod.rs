mod event_handler;
mod signal;

use anyhow::{Context, Result};
use mlua::{Lua, RegistryKey};
use serenity::all::Http;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex as StdMutex};

use crate::Data;
use crate::consts::DATA_DIR;
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

pub fn configure_lua_env(
    lua: &Lua,
    callbacks: Arc<StdMutex<BotCallbacks>>,
    http: Arc<Http>,
) -> anyhow::Result<()> {
    let globals = lua.globals();
    let skekbot = lua.create_table()?;

    let log_backend =
        lua.create_function(|_, (level, location, message): (String, String, String)| {
            match level.to_uppercase().as_str() {
                "ERROR" => tracing::error!("({}) {}", location, message),
                "WARN" => tracing::warn!("({}) {}", location, message),
                _ => tracing::info!("({}) {}", location, message),
            }
            Ok(())
        })?;
    skekbot.set("_log_backend", log_backend)?;

    let start_time = std::time::Instant::now();
    let uptime_helper = lua.create_function(move |_, ()| Ok(start_time.elapsed().as_secs()))?;
    skekbot.set("getUptime", uptime_helper)?;

    let sleep_helper = lua.create_async_function(|_, seconds: f64| async move {
        tokio::time::sleep(std::time::Duration::from_secs_f64(seconds)).await;
        Ok(())
    })?;
    skekbot.set("sleep", sleep_helper)?;

    let events = lua.create_table()?;
    events.set(
        "OnReady",
        create_signal(lua, Arc::clone(&callbacks), EventType::Ready)?,
    )?;
    events.set(
        "OnMessageCreate",
        create_signal(lua, Arc::clone(&callbacks), EventType::MessageCreate)?,
    )?;
    skekbot.set("Events", events)?;

    let rest = lua.create_table()?;
    let http_clone = Arc::clone(&http);

    let send_message =
        lua.create_async_function(move |_, (channel_id_str, content): (String, String)| {
            let http = Arc::clone(&http_clone);
            async move {
                let channel_id_u64 = channel_id_str.parse::<u64>().map_err(|_| {
                    mlua::Error::RuntimeError(
                        "invalid channel id: must be a numeric string".to_string(),
                    )
                })?;

                let channel_id = serenity::model::id::ChannelId::new(channel_id_u64);
                let message_builder = serenity::builder::CreateMessage::new().content(content);

                channel_id
                    .send_message(&http, message_builder)
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(format!("discord api error: {e}")))?;
                Ok(())
            }
        })?;

    rest.set("sendMessage", send_message)?;
    skekbot.set("Rest", rest)?;

    globals.set("Skekbot", skekbot)?;

    lua.load(
        r##"
        local function get_caller_info(stack_level)
            local source, line = debug.info(stack_level, "sl")
            if not source then return "unknown" end
            
            if string.sub(source, 1, 1) == "=" or string.sub(source, 1, 1) == "@" then
                source = string.sub(source, 2)
            end
            return source .. ":" .. tostring(line)
        end

        Skekbot.log = function(level, message)
            Skekbot._log_backend(level, get_caller_info(3), tostring(message))
        end

        print = function(...)
            local num_args = select("#", ...)
            local str = {}
            for i = 1, num_args do
                table.insert(str, tostring(select(i, ...)))
            end
            Skekbot._log_backend("INFO", get_caller_info(3), table.concat(str, "\t"))
        end
    "##,
    )
    .exec()?;

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

    configure_lua_env(&lua, Arc::clone(&data.lua_callbacks), http)?;

    let scripts_path = std::path::Path::new(DATA_DIR).join("luau").join("scripts");
    load_scripts(&lua, scripts_path)?;

    tracing::info!("luau reload complete!");

    Ok(())
}
