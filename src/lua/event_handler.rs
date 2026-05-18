use mlua::{Function, LuaSerdeExt, RegistryKey};
use std::{collections::HashMap, time::Duration};
use tokio::time::timeout;

use poise::serenity_prelude as serenity;

use crate::{
    Data, Error,
    lua::{BotCallbacks, modules::types::{LuaGuildMemberUpdate, LuaMessage, LuaUser}},
};

/// helper to safely lock callbacks and extract lua(u) functions from the registry
fn get_lua_callbacks(
    lua: &mlua::Lua,
    data: &Data,
    selector: impl FnOnce(&BotCallbacks) -> &HashMap<u64, RegistryKey>,
) -> anyhow::Result<Vec<Function>> {
    let funcs: Vec<Function> = {
        let cb = data
            .lua_callbacks
            .lock()
            .map_err(|_| anyhow::anyhow!("lua callbacks mutex poisoned"))?;
        
        let map = selector(&cb);
        
        map.values()
            .filter_map(|key| lua.registry_value::<Function>(key).ok())
            .collect()
    };

    Ok(funcs)
}

async fn handle_ready(data: &Data, data_about_bot: &serenity::all::Ready) -> Result<(), Error> {
    let lua = data.lua.lock().await;
    let funcs = get_lua_callbacks(&lua, data, |cb| &cb.ready_events)?;

    for func in funcs {
        let bot_name = data_about_bot.user.name.clone();
        let exec_future = func.call_async::<()>(bot_name);

        if let Err(e) = timeout(Duration::from_secs(5), exec_future).await {
            tracing::error!("luau OnReady script timed out or failed: {:?}", e);
        }
    }

    Ok(())
}

async fn handle_guild_member_update(
    data: &Data,
    event: &serenity::all::GuildMemberUpdateEvent,
) -> Result<(), Error> {
    let (funcs, lua_msg) = {
        let lua = data.lua.lock().await;
        let funcs = get_lua_callbacks(&lua, data, |cb| &cb.guild_member_update_events)?;

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

        (funcs, lua.to_value(&update_data)?)
    }; // lua lock drops here

    for func in funcs {
        let exec_future = func.call_async::<()>(lua_msg.clone());
        if let Err(e) = timeout(Duration::from_secs(5), exec_future).await {
            tracing::error!("Luau OnGuildMemberUpdate script timed out or failed: {:?}", e);
        }
    }

    Ok(())
}

async fn handle_message_create(
    data: &Data,
    new_message: &serenity::all::Message,
) -> Result<(), Error> {
    let (funcs, lua_msg) = {
        let lua = data.lua.lock().await;
        let funcs = get_lua_callbacks(&lua, data, |cb| &cb.message_create_events)?;

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

        (funcs, lua.to_value(&message_data)?)
    }; // lua lock drops here

    for func in funcs {
        let exec_future = func.call_async::<()>(lua_msg.clone());
        if let Err(e) = timeout(Duration::from_secs(5), exec_future).await {
            tracing::error!("Luau OnMessageCreate script timed out or failed: {:?}", e);
        }
    }

    Ok(())
}

#[allow(clippy::significant_drop_tightening)]
pub async fn lua_event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            handle_ready(data, data_about_bot).await?;
        }
        serenity::FullEvent::GuildMemberUpdate { event, .. } => {
            handle_guild_member_update(data, event).await?;
        }
        serenity::FullEvent::Message { new_message } => {
            handle_message_create(data, new_message).await?;
        }
        _ => {}
    }

    Ok(())
}