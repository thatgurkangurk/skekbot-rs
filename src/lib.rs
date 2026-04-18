pub mod commands;
mod config;
pub mod consts;
pub mod db;
pub mod event;
pub mod features;
pub mod models;
mod skekbot;
pub mod util;
pub mod web;

use moka::future::Cache;
use poise::serenity_prelude as serenity;
use sea_orm::DatabaseConnection;
use serenity::prelude::TypeMapKey;

pub use config::Config;
pub use skekbot::create_skekbot;

use crate::models::server;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    pub config: Config,
    pub db: DatabaseConnection,
    pub server_cache: Cache<u64, server::Model>,
}

impl TypeMapKey for Data {
    type Value = String;
}
