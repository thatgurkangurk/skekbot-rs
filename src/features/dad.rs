use crate::{Data, Error};
use ::serenity::all::Message;
use poise::serenity_prelude as serenity;

const IM: &[&str] = &["im", "i'm", "i’m"];
const MAX_LENGTH: usize = 75;

fn dad<'a>(content: &'a str, im_trigger_words: &[&str]) -> Option<&'a str> {
    let content = content.trim();
    if content.is_empty() {
        return None;
    }

    let lower = content.to_lowercase();

    let words: Vec<&str> = content.split_whitespace().collect();
    let lower_words: Vec<&str> = lower.split_whitespace().collect();

    if words.len() < 2 {
        return None;
    }

    let mut trigger_index = None;

    for i in 0..lower_words.len() {
        if lower_words[i] == "i" && lower_words.get(i + 1) == Some(&"am") {
            trigger_index = Some(i + 2);
        } else if im_trigger_words.contains(&lower_words[i]) {
            trigger_index = Some(i + 1);
        }
    }

    let start_word = trigger_index?;
    if start_word >= words.len() {
        return None;
    }

    // FIND byte index by walking split_whitespace again
    let mut current_word = 0;
    let mut byte_start = None;

    for (byte_index, _) in content.char_indices() {
        if content[byte_index..].starts_with(words[start_word]) {
            if current_word == start_word {
                byte_start = Some(byte_index);
                break;
            }
            current_word += 1;
        }
    }

    let byte_start = byte_start?;
    let remainder = &content[byte_start..];

    let end = remainder
        .find(['.', ',', '!', '?'])
        .unwrap_or(remainder.len());

    let result = remainder[..end].trim();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

async fn dad_joke(ctx: &serenity::Context, message: &Message, data: &Data) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

    let content = message.content.trim();
    if content.is_empty() {
        return Ok(());
    }

    let Some(name) = dad(content, IM) else {
        return Ok(());
    };

    let mut name = name
        .chars()
        .take(MAX_LENGTH)
        .collect::<String>()
        .trim()
        .to_string();

    if name.is_empty() {
        return Ok(());
    }

    name = crate::util::sanitise_pings(&name);

    message
        .reply(ctx, format!("Hi {}, I'm {}!", name, data.bot_name))
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
