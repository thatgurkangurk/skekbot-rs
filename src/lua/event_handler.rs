use mlua::Function;
use std::time::Duration;
use tokio::time::timeout;

use poise::serenity_prelude as serenity;

use crate::{Data, Error};

pub async fn lua_event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            let lua = data.lua.lock().await;

            let funcs: Vec<Function> = {
                let cb = data.lua_callbacks.lock().unwrap();
                cb.ready_events
                    .values()
                    .filter_map(|key| lua.registry_value::<Function>(key).ok())
                    .collect()
            };

            for func in funcs {
                let bot_name = data_about_bot.user.name.clone();
                let exec_future = func.call_async::<()>(bot_name);

                if let Err(e) = timeout(Duration::from_secs(5), exec_future).await {
                    tracing::error!("luau OnReady script timed out or failed: {:?}", e);
                }
            }
        }

        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot {
                return Ok(());
            }

            let lua = data.lua.lock().await;

            let funcs: Vec<Function> = {
                let cb = data.lua_callbacks.lock().unwrap();
                cb.message_create_events
                    .values()
                    .filter_map(|key| lua.registry_value::<Function>(key).ok())
                    .collect()
            };

            let lua_msg = lua
                .create_table()
                .unwrap_or_else(|_| lua.create_table().unwrap());
            let _ = lua_msg.set("content", new_message.content.clone());
            let _ = lua_msg.set("author", new_message.author.name.clone());
            let _ = lua_msg.set("channel_id", new_message.channel_id.get().to_string());

            for func in funcs {
                let exec_future = func.call_async::<()>(lua_msg.clone());
                if let Err(e) = timeout(Duration::from_secs(5), exec_future).await {
                    tracing::error!("Luau OnMessageCreate script timed out or failed: {:?}", e);
                }
            }
        }

        // ignore all other events
        _ => {}
    }

    Ok(())
}
