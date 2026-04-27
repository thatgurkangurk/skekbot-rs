use std::sync::{Arc, Mutex, atomic::Ordering};

use mlua::{Function, Lua};

use crate::lua::{BotCallbacks, EventType, NEXT_CONNECTION_ID};

pub fn create_signal(
    lua: &Lua,
    callbacks: &Arc<Mutex<BotCallbacks>>,
    event_type: EventType,
) -> mlua::Result<mlua::Table> {
    let signal = lua.create_table()?;

    let callbacks_clone = Arc::clone(callbacks);

    let connect = lua.create_function(move |lua, (_, func): (mlua::Value, Function)| {
        let key = lua.create_registry_value(func)?;
        let id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);

        // safe lock handling for callbacks
        {
            let mut cb = callbacks_clone.lock().map_err(|_| {
                mlua::Error::RuntimeError("BotCallbacks mutex poisoned during Connect".to_string())
            })?;

            match event_type {
                EventType::Ready => {
                    cb.ready_events.insert(id, key);
                }
                EventType::MessageCreate => {
                    cb.message_create_events.insert(id, key);
                }
            }
        } // lock drops here

        let connection = lua.create_table()?;
        let callbacks_disconnect = Arc::clone(&callbacks_clone);

        let disconnect = lua.create_function(move |_, _: mlua::Value| {
            let mut cb = callbacks_disconnect.lock().map_err(|_| {
                mlua::Error::RuntimeError(
                    "BotCallbacks mutex poisoned during Disconnect".to_string(),
                )
            })?;

            match event_type {
                EventType::Ready => {
                    cb.ready_events.remove(&id);
                }
                EventType::MessageCreate => {
                    cb.message_create_events.remove(&id);
                }
            }

            drop(cb);
            Ok(())
        })?;

        connection.set("Disconnect", disconnect)?;
        Ok(connection)
    })?;

    signal.set("Connect", connect)?;
    Ok(signal)
}
