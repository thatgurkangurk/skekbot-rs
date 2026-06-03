use std::sync::Arc;

use crate::{Context, Error, lua::get_loaded_scripts, lua::reload_scripts};

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

#[poise::command(slash_command, owners_only)]
/// list all currently loaded luau scripts
pub async fn list_loaded_scripts(ctx: Context<'_>) -> Result<(), Error> {
    let data = ctx.data();
    let lua_guard = data.lua.lock().await;

    match get_loaded_scripts(&lua_guard) {
        Ok((scripts, modules)) => {
            let mut response = String::new();

            if scripts.is_empty() && modules.is_empty() {
                ctx.say("no scripts are currently loaded").await?;
                return Ok(());
            }

            if !scripts.is_empty() {
                response.push_str("**executed scripts:**\n```\n");
                response.push_str(&scripts.join("\n"));
                response.push_str(
                    "\n
```\n",
                );
            }

            if !modules.is_empty() {
                response.push_str("**cached modules (`require`):**\n```\n");
                response.push_str(&modules.join("\n"));
                response.push_str(
                    "\n
```",
                );
            }

            if response.len() > 2000 {
                response.truncate(1996);
                response.push_str("...`");
            }

            ctx.say(response).await?;
        }
        Err(e) => {
            let err_msg = format!("failed to get loaded scripts: {e}");
            tracing::error!("{}", err_msg);
            ctx.say(err_msg).await?;
        }
    }

    Ok(())
}
