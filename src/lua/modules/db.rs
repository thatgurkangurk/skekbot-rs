use mlua::{Lua, LuaSerdeExt, Table};
use moka::future::Cache;
use sea_orm::DatabaseConnection;
use serenity::model::id::GuildId;

use crate::{db::get_or_create_server_table_cached, models::server};

pub fn setup(
    lua: &Lua,
    registry: &Table,
    db: &DatabaseConnection,
    server_cache: &Cache<u64, server::Model>,
) -> anyhow::Result<()> {
    let db_table = lua.create_table()?;
    let db_clone = db.clone();
    let cache_clone = server_cache.clone();

    let get_settings = lua.create_async_function(move |lua, server_id: String| {
        let db = db_clone.clone();
        let cache = cache_clone.clone();

        async move {
            let guild_id_u64 = server_id.parse::<u64>().map_err(|_| {
                mlua::Error::RuntimeError("Invalid Server ID: Must be a numeric string".to_string())
            })?;

            let guild_id = GuildId::new(guild_id_u64);

            let model = get_or_create_server_table_cached(&guild_id, &db, &cache)
                .await
                .map_err(|e| mlua::Error::RuntimeError(format!("Database Error: {e}")))?;

            let lua_value = lua.to_value(&model)?;
            Ok(lua_value)
        }
    })?;

    db_table.set("getOrCreateServerSettings", get_settings)?;
    registry.set("@skekbot/db", db_table)?;

    Ok(())
}
