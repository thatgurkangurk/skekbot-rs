use mlua::{Lua, Table};
use std::sync::{Arc, Mutex as StdMutex};

use crate::lua::{BotCallbacks, EventType, signal::create_signal};

pub fn setup(
    lua: &Lua,
    registry: &Table,
    callbacks: &Arc<StdMutex<BotCallbacks>>,
) -> anyhow::Result<()> {
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
    Ok(())
}
