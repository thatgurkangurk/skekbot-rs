use crate::{Context, Error};
use super::checks::is_ingame_moderator;

#[poise::command(slash_command, check = "is_ingame_moderator")]
/// ban someone from the game
pub async fn ban(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("yeah this does nothing yet").await?;

    Ok(())
}
