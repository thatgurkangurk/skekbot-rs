use crate::{Data, Error};
use ::serenity::all::Message;
use poise::serenity_prelude as serenity;

const IM_TRIGGERS: &[&str] = &["im", "i'm", "i’m", "i am"];
const DELIMITERS: &[char] = &['.', ',', '\n', '!', '?'];
const MAX_LENGTH: usize = 75;

fn get_dad_joke_name(content: &str) -> Option<String> {
    let content = content.trim();
    let lower = content.to_lowercase();
    let mut start_index: Option<usize> = None;

    // Collect char indices to safely check lookbehind for word boundaries
    let chars: Vec<(usize, char)> = lower.char_indices().collect();

    for (i, &(byte_idx, _)) in chars.iter().enumerate() {
        let is_boundary = i == 0 || chars[i - 1].1.is_whitespace();

        if is_boundary {
            for trigger in IM_TRIGGERS {
                if lower[byte_idx..].starts_with(trigger) {
                    let after_byte_idx = byte_idx + trigger.len(); // trigger.len() is byte length

                    let is_end_boundary = after_byte_idx == lower.len()
                        || lower[after_byte_idx..].starts_with(|c: char| c.is_whitespace());

                    if is_end_boundary {
                        start_index = Some(after_byte_idx);
                    }
                }
            }
        }
    }

    let mut start_byte = start_index?;

    if let Some(offset) = content[start_byte..].find(|c: char| !c.is_whitespace()) {
        start_byte += offset;
    } else {
        start_byte = content.len();
    }

    if start_byte >= content.len() {
        return None;
    }

    let remainder = &content[start_byte..];

    let end = remainder.find(DELIMITERS).unwrap_or(remainder.len());
    let result = remainder[..end].trim();

    if result.is_empty() {
        return None;
    }

    let truncated: String = result.chars().take(MAX_LENGTH).collect();
    let final_name = truncated.trim().to_string();

    if final_name.is_empty() {
        None
    } else {
        Some(final_name)
    }
}

async fn dad_joke(ctx: &serenity::Context, message: &Message, data: &Data) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

if let Some(guild_id) = message.guild_id {
        let num_id = guild_id.get();

        let server_table = data.server_cache
            .try_get_with(num_id, async {
                crate::db::get_or_create_server_table(&guild_id, &data.db).await
            })
            .await
            .map_err(|e| anyhow::anyhow!("Cache/DB error: {e}"))?; 

        if !server_table.dad_enabled {
            return Ok(());
        }
    }

    let Some(mut name) = get_dad_joke_name(&message.content) else {
        return Ok(());
    };

    name = crate::util::sanitise_pings(&name);

    message
        .reply(
            ctx,
            format!(
                "Hi {}, I'm {}!",
                name,
                ctx.cache.current_user().display_name()
            ),
        )
        .await?;

    Ok(())
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::Message { new_message } = event {
        dad_joke(ctx, new_message, data).await?;
    }
    Ok(())
}
