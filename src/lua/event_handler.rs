use mlua::{Function, LuaSerdeExt};
use std::time::Duration;
use tokio::time::timeout;

use poise::serenity_prelude as serenity;

use crate::{
    Data, Error,
    lua::modules::types::{LuaGuildMemberUpdate, LuaMessage, LuaUser},
};

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

        serenity::FullEvent::GuildMemberUpdate {
            old_if_available: _,
            new: _,
            event,
        } => {
            let (funcs, lua_msg) = {
                let lua = data.lua.lock().await;

                let funcs = {
                    let cb = data
                        .lua_callbacks
                        .lock()
                        .map_err(|_| anyhow::anyhow!("lua callbacks mutex poisoned"))?;

                    cb.guild_member_update_events
                        .values()
                        .filter_map(|key| lua.registry_value::<Function>(key).ok())
                        .collect::<Vec<_>>()
                };

                let user_data = LuaUser {
                    id: event.user.id.get().to_string(),
                    bot: event.user.bot,
                    global_name: event.user.global_name.clone(),
                    username: event.user.name.clone(),
                };

                let update_data = LuaGuildMemberUpdate {
                    guild_id: event.guild_id.get().to_string(),
                    user: user_data,
                    nick: event.nick.clone(),
                };

                let lua_value = lua.to_value(&update_data)?;

                (funcs, lua_value)
            };

            // 5. Execute all registered Lua callbacks asynchronously with a timeout
            for func in funcs {
                let exec_future = func.call_async::<()>(lua_msg.clone());
                if let Err(e) = timeout(Duration::from_secs(5), exec_future).await {
                    tracing::error!(
                        "Luau OnGuildMemberUpdate script timed out or failed: {:?}",
                        e
                    );
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
                    username: new_message.author.name.clone(),
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
