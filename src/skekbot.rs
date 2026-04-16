use crate::{Config, Data, Error, commands, event::event_handler_root, features::web::BotState};
use moka::future::Cache;
use poise::serenity_prelude as serenity;

use sea_orm::DatabaseConnection;
use tracing::error;

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {error:?}"),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {e}");
            }
        }
    }
}

pub async fn create_skekbot(config: &Config, db: &DatabaseConnection) -> anyhow::Result<BotState> {
    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::ping::ping(),
            commands::dad::dad(),
            commands::rock_paper_scissors::rock_paper_scissors(),
            commands::info::info(),
            commands::quote::quote(),
        ],
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler_root(ctx, event, framework, data))
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };

    let config_clone = config.clone();
    let db_clone = db.clone();

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    config: config_clone,
                    db: db_clone,
                    server_cache: Cache::builder()
                        .time_to_live(std::time::Duration::from_mins(5))
                        .build(),
                })
            })
        })
        .options(options)
        .build();

    // 1. Create the client
    let client = serenity::ClientBuilder::new(&config.bot.token, intents)
        .framework(framework)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create client: {e}"))?;

    let bot_state = BotState::new(client, config);

    Ok(bot_state)
}
