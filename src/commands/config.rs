use poise::CreateReply;
use serenity::all::{Colour, CreateEmbed, Timestamp};

use crate::{Context, Error};

#[poise::command(
    slash_command,
    required_bot_permissions = "MANAGE_GUILD",
    required_permissions = "MANAGE_GUILD",
    guild_only,
    guild_cooldown = 2
)]
/// refresh server config
pub async fn refresh_config(ctx: Context<'_>) -> Result<(), Error> {
    let bot_user = ctx.cache().current_user().clone();

    #[allow(clippy::unwrap_used)] // guild_only protects us
    let current_guild_id = ctx.guild_id().unwrap();

    ctx.data()
        .server_cache
        .invalidate(&current_guild_id.get())
        .await;

    let embed = CreateEmbed::new()
        .title("skekbot-rs")
        .description("successfully refreshed the server settings")
        .colour(Colour::from_rgb(236, 253, 245))
        .timestamp(Timestamp::now())
        .thumbnail(bot_user.face());

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}
