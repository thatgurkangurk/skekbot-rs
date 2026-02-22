use crate::{Context, Error};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DadJokeResponse {
    joke: String,
}

async fn fetch_dad_joke() -> Result<DadJokeResponse, Error> {
    let response = reqwest::get("https://api.gurkz.me/api/dad-joke")
        .await?
        .json::<DadJokeResponse>()
        .await?;

    Ok(response)
}

#[poise::command(
    slash_command,
)]
/// gives a (not) very funny dad joke
pub async fn dad(ctx: Context<'_>) -> Result<(), Error> {
    let joke = match fetch_dad_joke().await {
        Ok(j) => j.joke,
        Err(_) => {
            ctx.say("the dad joke was so bad that i'm not even going to say it...")
                .await?;
            return Ok(());
        }
    };

    ctx.say(format!("here's one: {}", joke)).await?;

    Ok(())
}
