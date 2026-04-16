use sea_orm::{Database, DatabaseConnection};
use std::fs;
use std::path::Path;
use crate::consts::DATA_DIR;

pub async fn create_db() -> anyhow::Result<DatabaseConnection> {
    let db_path = Path::new(DATA_DIR).join("skekbot.sqlite3");
    
    let db_path_str = db_path.to_string_lossy();
    
    let db_url = format!("sqlite:{db_path_str}?mode=rwc");

    if !db_path.exists() {
        println!("WARNING: db does not exist at '{db_path_str}'. creating it now...");
        
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