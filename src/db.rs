use crate::Config;
use crate::consts::DATA_DIR;
use crate::models::server;
use moka::future::Cache;
use sea_orm::sea_query::OnConflict;
use sea_orm::sea_query::prelude::rust_decimal::prelude::ToPrimitive;
use sea_orm::{Database, DatabaseConnection, DbErr, EntityTrait, Set};
use serenity::all::GuildId;
use std::fs;
use std::path::Path;
use tracing::warn;

async fn create_db_sqlite() -> anyhow::Result<DatabaseConnection> {
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

    Ok(db)
}

async fn create_db_external(config: &Config) -> anyhow::Result<DatabaseConnection> {
    if let Some(db_config) = &config.db {
        let db = Database::connect(&db_config.uri).await?;

        return Ok(db);
    }

    anyhow::bail!("no db connection config was provided");
}

pub async fn create_db(config: &Config) -> anyhow::Result<DatabaseConnection> {
    let db = match config.db {
        Some(_) => create_db_external(config).await?,
        None => create_db_sqlite().await?,
    };

    db.get_schema_registry("skekbot_rs::models::*")
        .sync(&db)
        .await?;

    Ok(db)
}

pub async fn get_or_create_server_table_cached(
    guild_id: &GuildId,
    db: &DatabaseConnection,
    cache: &Cache<u64, server::Model>,
) -> anyhow::Result<server::Model> {
    let server_table = cache
        .try_get_with(guild_id.get(), async {
            get_or_create_server_table(guild_id, db).await
        })
        .await
        .map_err(|e| anyhow::anyhow!("Cache/DB error: {e}"))?;

    Ok(server_table)
}

/// immediately fetches the server table from the db
///
/// for most purposes please use [`get_or_create_server_table_cached`] instead
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

    let new_server = server::ActiveModel {
        id: Set(num_guild_id),
        ..Default::default()
    };

    let insert_result = server::Entity::insert(new_server)
        .on_conflict(
            OnConflict::column(server::Column::Id)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await;

    match insert_result {
        Ok(_) | Err(DbErr::RecordNotInserted) => {} // either success, great :3! or a conflict, thats fine, ignore it
        Err(e) => return Err(e.into()),             // db error happened, bubble it
    }

    let server = server::Entity::find_by_id(num_guild_id)
        .one(db)
        .await?
        .ok_or_else(|| {
            anyhow::anyhow!("critical: server was not found immediately after upsert")
        })?;

    Ok(server)
}
