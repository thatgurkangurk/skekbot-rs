#![allow(clippy::needless_for_each)]
use serenity::Client;
use serenity::all::ShardManager;
use std::sync::Arc;

use crate::Config;

#[derive(Clone)]
pub struct BotState {
    pub shard_manager: Arc<ShardManager>,
    pub http: Arc<serenity::http::Http>,
    pub config: Config,
}

impl BotState {
    #[must_use]
    pub fn new(client: &Client, config: &Config) -> Self {
        let config_clone = config.clone();
        Self {
            shard_manager: client.shard_manager.clone(),
            http: client.http.clone(),
            config: config_clone,
        }
    }
}
