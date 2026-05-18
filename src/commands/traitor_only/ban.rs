use crate::{Context, Error};

#[poise::command(slash_command)]
/// ban someone from the game
pub async fn ban(ctx: Context<'_>) -> Result<(), Error> {
    ctx.reply("yeah this does nothing yet").await?;

    Ok(())
}
