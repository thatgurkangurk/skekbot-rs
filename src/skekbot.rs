use std::{path::Path, sync::Arc};

use crate::{
    Config, Data, Error, commands,
    consts::DATA_DIR,
    event::event_handler_root,
    lua::{BotCallbacks, configure_lua_env, load_scripts},
    web::BotState,
};
use anyhow::Context;
use mlua::Lua;
use moka::future::Cache;
use poise::serenity_prelude as serenity;

use crate::StdMutex;
use ::serenity::Client;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex as TokioMutex;
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

pub async fn create_skekbot(
    config: &Config,
    db: &DatabaseConnection,
) -> anyhow::Result<(Client, BotState)> {
    let intents = serenity::GatewayIntents::GUILDS
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::MESSAGE_CONTENT;

    // ==========================================
    // initialise luau early (pre-framework)
    // ==========================================
    let server_cache = Cache::builder()
        .time_to_live(std::time::Duration::from_secs(300)) // 5 mins
        .build();

    tracing::info!("initialising luau...");
    let lua = Arc::new(TokioMutex::new(Lua::new()));
    let lua_callbacks = Arc::new(StdMutex::new(BotCallbacks::default()));

    {
        let lua_lock = lua.lock().await;

        // we need a dummy/temporary Http client because we aren't in poise yet
        let temp_http = Arc::new(serenity::all::Http::new(config.bot.token.as_ref()));

        configure_lua_env(
            &lua_lock,
            &Arc::clone(&lua_callbacks),
            &temp_http, // use temp http here for type generation
            &server_cache,
            db,
            &Path::new(DATA_DIR).join("luau").join("modules"),
        )
        .context("failed to configure the luau global environment")?;

        let scripts_path = Path::new(DATA_DIR).join("luau").join("scripts");
        load_scripts(&lua_lock, &scripts_path).context("failed to load luau scripts")?;
    }

    let config_clone = config.clone();
    let db_clone = db.clone();
    let lua_clone = Arc::clone(&lua);
    let callbacks_clone = Arc::clone(&lua_callbacks);
    let cache_clone = server_cache.clone();

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::ping::ping(),
            commands::dad::dad(),
            commands::rock_paper_scissors::rock_paper_scissors(),
            commands::info::info(),
            commands::quote::quote(),
            commands::config::refresh_config(),
            commands::luau::reload(),
        ],
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler_root(ctx, event, framework, data))
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                {
                    let lua_lock = lua_clone.lock().await;
                    let rest_mod = crate::lua::modules::rest::setup(&lua_lock, &ctx.http)?;
                    rest_mod.register(&lua_lock.named_registry_value("__SKEKBOT_REGISTRY")?)?;
                }

                Ok(Data {
                    config: config_clone,
                    db: db_clone,
                    server_cache: cache_clone,
                    lua: lua_clone,
                    lua_callbacks: callbacks_clone,
                })
            })
        })
        .options(options)
        .build();

    let client = serenity::ClientBuilder::new(&config.bot.token, intents)
        .framework(framework)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create client: {e}"))?;

    let bot_state = BotState::new(&client, config);

    Ok((client, bot_state))
}
