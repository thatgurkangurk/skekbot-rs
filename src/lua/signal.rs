use std::sync::{Arc, Mutex, atomic::Ordering};

use mlua::{Function, Lua};

use crate::lua::{BotCallbacks, EventType, NEXT_CONNECTION_ID};

pub fn create_signal(
    lua: &Lua,
    callbacks: Arc<Mutex<BotCallbacks>>,
    event_type: EventType,
) -> mlua::Result<mlua::Table> {
    let signal = lua.create_table()?;

    let callbacks_clone = Arc::clone(&callbacks);

    // the rust signature is (Value, Function) because `obj:Connect(func)`
    // secretly passes `obj` as the first argument. we ignore it with `_`.
    let connect = lua.create_function(move |lua, (_, func): (mlua::Value, Function)| {
        let key = lua.create_registry_value(func)?;
        let id = NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed);

        // insert the event into our map
        let mut cb = callbacks_clone.lock().unwrap();
        match event_type {
            EventType::Ready => {
                cb.ready_events.insert(id, key);
            }
            EventType::MessageCreate => {
                cb.message_create_events.insert(id, key);
            }
        }
        drop(cb); // drop the lock quickly

        // create the object to return to lua
        let connection = lua.create_table()?;
        let callbacks_disconnect = Arc::clone(&callbacks_clone);

        // the signature is Value because `connection:Disconnect()` passes `connection`
        let disconnect = lua.create_function(move |_, _: mlua::Value| {
            let mut cb = callbacks_disconnect.lock().unwrap();
            match event_type {
                EventType::Ready => {
                    cb.ready_events.remove(&id);
                }
                EventType::MessageCreate => {
                    cb.message_create_events.remove(&id);
                }
            }
            Ok(())
        })?;

        connection.set("Disconnect", disconnect)?;
        Ok(connection)
    })?;

    signal.set("Connect", connect)?;
    Ok(signal)
}
