use mlua::Lua;
use std::sync::{Arc, Mutex as StdMutex};

use crate::lua::builder::ModuleBuilder;
use crate::lua::{BotCallbacks, EventType, signal::create_signal};

pub fn setup(lua: &Lua, callbacks: &Arc<StdMutex<BotCallbacks>>) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Events")?;
    builder.use_module("types", "Types");

    builder
        .add_type_declaration("export type Connection = { Disconnect: (self: Connection) -> () }");

    builder.add_type_declaration(
        "export type Signal<T> = { Connect: (self: Signal<T>, callback: (T) -> ()) -> () }",
    );

    builder.add_value(
        "OnReady",
        "Signal<string>",
        create_signal(lua, &Arc::clone(callbacks), EventType::Ready)?,
    )?;

    builder.add_value(
        "OnMessageCreate",
        "Signal<Types.Message>",
        create_signal(lua, &Arc::clone(callbacks), EventType::MessageCreate)?,
    )?;

    builder.add_value(
        "OnGuildMemberUpdate",
        "Signal<Types.GuildMemberUpdate>",
        create_signal(lua, &Arc::clone(callbacks), EventType::GuildMemberUpdate)?,
    )?;

    Ok(builder)
}
