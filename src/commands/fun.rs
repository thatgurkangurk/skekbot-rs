use rand::seq::IndexedRandom;
use std::time::Duration;
use tokio::time::sleep;

use crate::{Context, Data, Error};

const EIGHT_BALL_ANSWERS: &[&str] = &[
    "It is certain",
    "It is decidedly so",
    "Without a doubt",
    "Yes definitely",
    "You may rely on it",

    "As I see it, yes",
    "Most likely",
    "Outlook good",
    "Yes",
    "Signs point to yes",
    
    "Reply hazy, try again",
    "Ask again later",
    "Better not tell you now",
    "Cannot predict now",
    "Concentrate and ask again",

    "Don't count on it",
    "My reply is no",
    "My sources say no",
    "Outlook not so good",
    "Very doubtful",
];

#[poise::command(slash_command, rename = "8ball")]
/// ask the magic 8 ball a question
async fn magic_8ball(
    ctx: Context<'_>,
    #[description = "What do you want to ask?"] question: String,
) -> Result<(), Error> {
    let reply_handle = ctx.say("🎱 Shaking the magic 8 ball...").await?;

    sleep(Duration::from_secs(3)).await;

    let answer = {
        let mut rng = rand::rng();
        *EIGHT_BALL_ANSWERS.choose(&mut rng).unwrap()
    };

    let author_name = ctx
        .author_member()
        .await
        .map(|member| member.display_name().to_string())
        .unwrap_or_else(|| ctx.author().name.clone());

    let final_content = format!("**{} asks:** {}\n{}", author_name, question, answer);

    reply_handle
        .edit(ctx, poise::CreateReply::default().content(final_content))
        .await?;

    Ok(())
}

pub fn fun_commands() -> Vec<poise::Command<Data, Error>> {
    vec![magic_8ball()]
}
