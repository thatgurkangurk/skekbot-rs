pub mod commands;
mod config;
pub mod consts;
pub mod db;
pub mod event;
pub mod features;
pub mod lua;
pub mod models;
mod skekbot;
pub mod util;
pub mod web;

use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex as TokioMutex;

use mlua::Lua;
use moka::future::Cache;
use poise::serenity_prelude as serenity;
use sea_orm::DatabaseConnection;
use serenity::prelude::TypeMapKey;

pub use config::Config;
pub use skekbot::create_skekbot;

use crate::{lua::BotCallbacks, models::server};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    pub config: Config,
    pub db: DatabaseConnection,
    pub server_cache: Cache<u64, server::Model>,

    pub lua: Arc<TokioMutex<Lua>>,
    pub lua_callbacks: Arc<StdMutex<BotCallbacks>>,
}

impl TypeMapKey for Data {
    type Value = String;
}
