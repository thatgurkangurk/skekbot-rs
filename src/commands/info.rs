use poise::CreateReply;
use serenity::all::{Colour, CreateEmbed, Timestamp};

use crate::{Context, Error, consts};

#[poise::command(slash_command)]
/// get misc info about the bot
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let bot_user = ctx.cache().current_user().clone();

    let embed = CreateEmbed::new()
        .title("skekbot-rs")
        .field("version", consts::VERSION, true)
        .field("authors", consts::AUTHORS_RAW.replace(':', ", "), true)
        .field("repository", consts::REPOSITORY, false)
        .colour(Colour::from_rgb(236, 253, 245))
        .timestamp(Timestamp::now())
        .thumbnail(bot_user.face());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
