use mlua::{Function, LuaSerdeExt};
use std::time::Duration;
use tokio::time::timeout;

use poise::serenity_prelude as serenity;

use crate::{Data, Error, lua::modules::types::{LuaMessage, LuaUser}};

#[allow(clippy::significant_drop_tightening)]
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
                let cb = data
                    .lua_callbacks
                    .lock()
                    .map_err(|_| anyhow::anyhow!("lua callbacks mutex poisoned"))?;

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
            let (funcs, lua_msg) = {
                let lua = data.lua.lock().await;

                let func = {
                    let cb = data
                        .lua_callbacks
                        .lock()
                        .map_err(|_| anyhow::anyhow!("lua callbacks mutex poisoned"))?;
                    cb.message_create_events
                        .values()
                        .filter_map(|key| lua.registry_value::<Function>(key).ok())
                        .collect::<Vec<_>>()
                };

                let author_data = LuaUser {
                    id: new_message.author.id.get().to_string(),
                    bot: new_message.author.bot,
                    global_name: new_message.author.global_name.clone(),
                    username: new_message.author.name.clone()
                };

                let message_data = LuaMessage {
                    id: new_message.id.get().to_string(),
                    content: new_message.content.clone(),
                    author: author_data,
                    channel_id: new_message.channel_id.get().to_string(),
                    guild_id: new_message.guild_id.map(|id| id.get().to_string()),
                };

                let lua_value = lua.to_value(&message_data)?;

                (func, lua_value)
            };

            for func in funcs {
                let exec_future = func.call_async::<()>(lua_msg.clone());
                if let Err(e) = timeout(Duration::from_secs(5), exec_future).await {
                    tracing::error!("Luau OnMessageCreate script timed out or failed: {:?}", e);
                }
            }
        }

        _ => {}
    }

    Ok(())
}
