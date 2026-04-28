use mlua::Lua;
use std::sync::{Arc, Mutex as StdMutex};

use crate::lua::builder::ModuleBuilder;
use crate::lua::{BotCallbacks, EventType, signal::create_signal};

pub fn setup(lua: &Lua, callbacks: &Arc<StdMutex<BotCallbacks>>) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Events")?;

    builder
        .add_type_declaration("export type Connection = { Disconnect: (self: Connection) -> () }");

    builder.add_type_declaration(
        "export type Signal<T> = { Connect: (self: Signal<T>, callback: (T) -> ()) -> () }",
    );

    builder.add_type_declaration(
        "export type Message = { content: string, author: string, channel_id: string, guild_id: string? }"
    );

    builder.add_value(
        "OnReady",
        "Signal<string>",
        create_signal(lua, &Arc::clone(callbacks), EventType::Ready)?,
    )?;

    builder.add_value(
        "OnMessageCreate",
        "Signal<Message>",
        create_signal(lua, &Arc::clone(callbacks), EventType::MessageCreate)?,
    )?;

    Ok(builder)
}
