use poise::CreateReply;
use serenity::all::{Colour, CreateEmbed, Timestamp};

use crate::{Context, Error};

#[poise::command(slash_command)]
/// get misc info about the bot
pub async fn info(ctx: Context<'_>) -> Result<(), Error> {
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS").replace(':', ", ");
    let repository = env!("CARGO_PKG_REPOSITORY");

    let bot_user = ctx.cache().current_user().clone();

    let embed = CreateEmbed::new()
        .title("skekbot-rs")
        .field("version", version, true)
        .field("authors", authors, true)
        .field("repository", repository, false)
        .colour(Colour::from_rgb(236, 253, 245))
        .timestamp(Timestamp::now())
        .thumbnail(bot_user.face());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
