use std::sync::Arc;

use crate::{Context, Error, lua::reload_scripts};

#[poise::command(slash_command, owners_only)]
/// reload luau scripts
pub async fn reload(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let http = Arc::clone(&ctx.serenity_context().http);

    match reload_scripts(data, http).await {
        Ok(()) => {
            ctx.say("scripts reloaded successfully!").await?;
        }
        Err(e) => {
            let err_msg = format!("reload failed: {e}");
            tracing::error!("{}", err_msg);
            ctx.say(err_msg).await?;
        }
    }

    Ok(())
}
