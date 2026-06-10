pub mod ban;

mod checks {
    use crate::{Context, Error};

    pub async fn is_ingame_moderator(ctx: Context<'_>) -> Result<bool, Error> {
        let traitor_config = ctx.data().config.traitor.as_ref()
            .ok_or("no config existed")?;

        let current_guild_id = ctx.guild_id()
            .ok_or("this command must be run in a server, not in DMs")?;

        if current_guild_id != traitor_config.discord_server_id {
            return Err("this command can only be used in the Traitor server".into());
        }

        let member = ctx.author_member().await
            .ok_or("couldn't fetch server member profile")?;

        let has_mod_role = member.roles.iter()
            .any(|role_id| traitor_config.moderator_role_ids.contains(role_id));

        if has_mod_role {
            Ok(true)
        } else {
            Err("you don't have permission to do that".into())
        }
    }
}