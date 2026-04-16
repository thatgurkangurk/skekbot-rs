use crate::consts::DATA_DIR;
use crate::models::server;
use sea_orm::sea_query::prelude::rust_decimal::prelude::ToPrimitive;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use serenity::all::GuildId;
use std::fs;
use std::path::Path;
use tracing::warn;

pub async fn create_db() -> anyhow::Result<DatabaseConnection> {
    let db_path = Path::new(DATA_DIR).join("skekbot.sqlite3");

    let db_path_str = db_path.to_string_lossy();

    let db_url = format!("sqlite:{db_path_str}?mode=rwc");

    if !db_path.exists() {
        warn!("db does not exist at '{db_path_str}'. creating it now...");

        let data_dir_path = Path::new(DATA_DIR);
        if !data_dir_path.exists() {
            fs::create_dir_all(data_dir_path)?;
        }

        // no need to create a file, sqlite does that with ?mode=rwc
    }

    let db = Database::connect(&db_url).await?;

    db.get_schema_registry("skekbot_rs::models::*")
        .sync(&db)
        .await?;

    Ok(db)
}

pub async fn get_or_create_server_table(
    guild_id: &GuildId,
    db: &DatabaseConnection,
) -> anyhow::Result<server::Model> {
    let num_guild_id = guild_id.get();

    let Some(num_guild_id) = num_guild_id.to_i64() else {
        return Err(anyhow::anyhow!(
            "{num_guild_id} could not be converted to i64"
        ));
    };

    let maybe_server = server::Entity::find_by_id(num_guild_id).one(db).await?;

    if let Some(server) = maybe_server {
        Ok(server)
    } else {
        let new_server = server::ActiveModel {
            id: Set(num_guild_id),
            ..Default::default()
        };

        let created = new_server.insert(db).await?;
        Ok(created)
    }
}
