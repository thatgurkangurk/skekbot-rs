use crate::{Data, Error};
use ::serenity::all::Message;
use poise::serenity_prelude as serenity;

const IM: &[&str] = &["im", "i'm", "i’m"];
const MAX_LENGTH: usize = 75;

async fn dad_joke(ctx: &serenity::Context, message: &Message) -> Result<(), Error> {
    if message.author.bot {
        return Ok(());
    }

    let content = &message.content;
    let lower = content.to_lowercase();

    let mut start_index = None;

    for (i, word) in lower.split_whitespace().enumerate() {
        if word == "i" {
            if lower.split_whitespace().nth(i + 1) == Some("am") {
                if let Some(pos) = lower.find("am") {
                    start_index = Some(pos + 3); // "am "
                    break;
                }
            }
        } else if IM.contains(&word) {
            if let Some(pos) = lower.find(word) {
                start_index = Some(pos + word.len() + 1);
                break;
            }
        }
    }

    let start = match start_index {
        Some(i) if i < content.len() => i,
        _ => return Ok(()),
    };

    let mut name = content[start..]
        .split(['.', ','])
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .take(MAX_LENGTH)
        .collect::<String>();

    if name.is_empty() {
        return Ok(());
    }

    name = crate::util::sanitise_pings(&name);

    message
        .reply(ctx, format!("Hi {}, I'm Skekbot!", name))
        .await?;

    Ok(())
}

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            dad_joke(ctx, new_message).await?
        }
        _ => {}
    }
    Ok(())
}
