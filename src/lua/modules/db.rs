use mlua::{Lua, LuaSerdeExt};
use moka::future::Cache;
use sea_orm::DatabaseConnection;
use serenity::model::id::GuildId;

use crate::lua::builder::ModuleBuilder;
use crate::{db::get_or_create_server_table_cached, models::server};

pub fn setup(
    lua: &Lua,
    db: &DatabaseConnection,
    server_cache: &Cache<u64, server::Model>,
) -> anyhow::Result<ModuleBuilder> {
    let mut builder = ModuleBuilder::new(lua, "Db")?;

    let db_clone = db.clone();
    let cache_clone = server_cache.clone();

    builder.add_type_declaration(
        r"
export type ServerSettings = {
    guild_id: string,
    prefix: string,
    logging_channel: string?,
}
    ",
    );

    builder.add_async_function(
        lua,
        "getOrCreateServerSettings",
        "(server_id: string) -> ServerSettings",
        move |lua, server_id: String| {
            let db = db_clone.clone();
            let cache = cache_clone.clone();

            async move {
                let guild_id_u64 = server_id.parse::<u64>().map_err(|_| {
                    mlua::Error::RuntimeError(
                        "Invalid Server ID: Must be a numeric string".to_string(),
                    )
                })?;

                let guild_id = GuildId::new(guild_id_u64);

                let model = get_or_create_server_table_cached(&guild_id, &db, &cache)
                    .await
                    .map_err(|e| mlua::Error::RuntimeError(format!("Database Error: {e}")))?;

                let lua_value = lua.to_value(&model)?;
                Ok(lua_value)
            }
        },
    )?;

    Ok(builder)
}
