use mlua::Lua;
use serenity::all::Http;
use serenity::model::id::ChannelId;
use std::sync::Arc;

use crate::lua::builder::ModuleBuilder;

// Make sure to import your builder! Adjust this path if it's stored elsewhere.

pub fn setup(lua: &Lua, http: &Arc<Http>) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Rest")?;
    let http_clone = Arc::clone(http);

    builder.add_async_function(
        lua,
        "sendMessage",
        "(channel_id: string, content: string) -> ()", // The Luau signature!
        move |_, (channel_id_str, content): (String, String)| {
            let http = Arc::clone(&http_clone);
            async move {
                let channel_id_u64 = channel_id_str.parse::<u64>().map_err(|_| {
                    mlua::Error::RuntimeError(
                        "invalid channel id: must be a numeric string".to_string(),
                    )
                })?;

                let channel_id = ChannelId::new(channel_id_u64);
                let message_builder = serenity::builder::CreateMessage::new().content(content);

                channel_id
                    .send_message(&http, message_builder)
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(format!("discord api error: {e}")))?;

                Ok(())
            }
        },
    )?;

    Ok(builder)
}
