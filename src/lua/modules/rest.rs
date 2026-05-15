use ::serenity::model::{
    channel::{MessageReference, MessageReferenceKind},
    id::{ChannelId, MessageId},
};
use mlua::Lua;
use poise::serenity_prelude as serenity;
use std::sync::Arc;

use crate::lua::{builder::ModuleBuilder, modules::types::LuaMessage};

pub fn setup(lua: &Lua, http: &Arc<serenity::Http>) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Rest")?;
    builder.use_module("types", "Types");

    let http_send = Arc::clone(http);

    builder.add_async_function(
        lua,
        "sendMessage",
        "(channel_id: string, content: string) -> ()",
        move |_, (channel_id_str, content): (String, String)| {
            let http = Arc::clone(&http_send);
            async move {
                let channel_id_u64 = channel_id_str.parse::<u64>().map_err(|_| {
                    mlua::Error::RuntimeError(
                        "invalid channel id: must be a numeric string".to_string(),
                    )
                })?;

                let channel_id = serenity::ChannelId::new(channel_id_u64);
                let message_builder = serenity::CreateMessage::new().content(content);

                channel_id
                    .send_message(&http, message_builder)
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(format!("discord api error: {e}")))?;

                Ok(())
            }
        },
    )?;

    let http_reply = Arc::clone(http);

    builder.add_async_function(
        lua,
        "replyToMessage",
        "(message: Types.Message, content: string) -> ()",
        move |_, (message, content): (LuaMessage, String)| {
            let http = Arc::clone(&http_reply);
            async move {
                let channel_id_u64 = message.channel_id.parse::<u64>().map_err(|_| {
                    mlua::Error::RuntimeError(
                        "invalid channel id: must be a numeric string".to_string(),
                    )
                })?;

                let message_id_u64 = message.id.parse::<u64>().map_err(|_| {
                    mlua::Error::RuntimeError(
                        "invalid message id: must be a numeric string".to_string(),
                    )
                })?;

                let channel_id = ChannelId::new(channel_id_u64);
                let message_id = MessageId::new(message_id_u64);

                let message_builder = serenity::builder::CreateMessage::new()
                    .content(content)
                    .reference_message(
                        MessageReference::new(MessageReferenceKind::Default, channel_id)
                            .message_id(message_id)
                            .fail_if_not_exists(true),
                    );

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
