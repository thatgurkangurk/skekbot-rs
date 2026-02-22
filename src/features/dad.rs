use crate::{Data, Error};
use poise::serenity_prelude as serenity;

const IM: &[&str] = &["im", "i'm", "i’m"];
const MAX_LENGTH: usize = 75;

pub async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot {
                return Ok(());
            }

            let content = &new_message.content;
            let lower_content = content.to_lowercase();

            let mut start_index: Option<usize> = None;

            let words: Vec<&str> = lower_content.split(" ").collect();

            for (i, word) in words.iter().enumerate() {
                if *word == "i" {
                    if let Some(next) = words.get(i + 1) {
                        if *next == "am" {
                            if let Some(pos) = lower_content.find("am") {
                                start_index = Some(pos + 2 + 1); // "am".len() + space
                                break;
                            }
                        }
                    }
                }

                if IM.contains(word) {
                    if let Some(pos) = lower_content.find(word) {
                        start_index = Some(pos + word.len() + 1);
                        break;
                    }
                }
            }

            let start_index = match start_index {
                Some(i) if i < content.len() => i,
                _ => return Ok(()),
            };

            let mut name = content[start_index..].to_string();

            if let Some(idx) = name.find('.') {
                name.truncate(idx);
            }
            if let Some(idx) = name.find(',') {
                name.truncate(idx);
            }

            name.truncate(MAX_LENGTH);
            let name = name.trim();

            if name.is_empty() {
                return Ok(());
            }

            let name = crate::util::sanitise_pings(name);

            new_message.reply(ctx, format!("Hi {}, I'm Skekbot!", name)).await?;
        }
        _ => {}
    }
    Ok(())
}
