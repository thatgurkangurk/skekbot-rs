use mlua::{Lua, Table};
use serenity::all::Http;
use serenity::model::id::ChannelId;
use std::sync::Arc;

pub fn setup(lua: &Lua, registry: &Table, http: &Arc<Http>) -> anyhow::Result<()> {
    let rest_table = lua.create_table()?;
    let http_clone = Arc::clone(http);

    let send_message =
        lua.create_async_function(move |_, (channel_id_str, content): (String, String)| {
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
        })?;

    rest_table.set("sendMessage", send_message)?;
    registry.set("@skekbot/rest", rest_table)?;

    Ok(())
}
