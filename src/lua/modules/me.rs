use std::sync::Arc;

use mlua::Lua;
use mlua::LuaSerdeExt;
use poise::serenity_prelude as serenity;

use crate::lua::{builder::ModuleBuilder, modules::types::LuaUser};

pub fn setup(lua: &Lua, http: &Arc<serenity::Http>) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Me")?;
    builder.use_module("types", "Types");

    let http_get_current_user = Arc::clone(http);

    builder.add_async_function(lua, "getCurrentUser", "() -> Types.User", move |lua, ()| {
        let http = Arc::clone(&http_get_current_user);

        async move {
            let result = http
                .get_current_user()
                .await
                .map_err(|_| mlua::Error::RuntimeError("failed to get myself".to_string()))?;

            let user_data = LuaUser {
                global_name: result.global_name.clone(),
                id: result.id.to_string(),
                username: result.name.clone(),
                bot: result.bot,
            };

            lua.to_value(&user_data)
                .map_err(|_| mlua::Error::RuntimeError("failed to convert user data".to_string()))
        }
    })?;

    Ok(builder)
}
