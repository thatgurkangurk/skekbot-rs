use std::env;

use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("Logged in as {}", ready.user.tag());
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("expected .env to load");

    let token = env::var("DISCORD_TOKEN")
        .expect("expected the DISCORD_TOKEN environment variable to exist");
    
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("error creating client");

    if let Err(why) = client.start().await {
        println!("client error: {why:?}");
    }
}