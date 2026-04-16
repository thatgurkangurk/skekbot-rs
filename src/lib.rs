pub mod commands;
mod config;
pub mod consts;
pub mod db;
pub mod event;
pub mod features;
pub mod models;
mod skekbot;
pub mod util;

use poise::serenity_prelude as serenity;
use serenity::prelude::TypeMapKey;

pub use config::Config;
pub use skekbot::create_skekbot;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    pub config: Config,
}

impl TypeMapKey for Data {
    type Value = String;
}
