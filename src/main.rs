mod commands;
mod consts;
mod event;
mod features;
mod util;

use ::serenity::prelude::TypeMapKey;
use console::style;
use poise::serenity_prelude as serenity;
use std::env;

use crate::event::event_handler_root;
use crate::features::web::{self, BotState};
use crate::util::validate_token;

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {}

impl TypeMapKey for Data {
    type Value = String;
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {error:?}"),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {e}");
            }
        }
    }
}

fn print_startup_info() {
    let lines = [
        format!("skekbot-rs {} by gurkan", consts::VERSION),
        "MPL 2.0 license".to_string(),
        consts::VERSION.to_string(),
    ];

    let content_width = lines
        .iter()
        .map(std::string::String::len)
        .max()
        .unwrap_or(0);
    let total_width = content_width + 4;

    println!();

    // top border
    println!(
        "{}{}{}",
        style("╔").cyan().bold(),
        style("═".repeat(total_width)).cyan().bold(),
        style("╗").cyan().bold(),
    );

    for (i, line) in lines.iter().enumerate() {
        let padding = content_width - line.len();
        let left = padding / 2;
        let right = padding - left;

        let content = if i == 0 {
            format!(
                "{}{}{}",
                " ".repeat(left),
                style(line).bold(),
                " ".repeat(right),
            )
        } else {
            format!("{}{}{}", " ".repeat(left), line, " ".repeat(right),)
        };

        println!(
            "{}  {}  {}",
            style("║").cyan().bold(),
            content,
            style("║").cyan().bold(),
        );
    }

    // bottom border
    println!(
        "{}{}{}",
        style("╚").cyan().bold(),
        style("═".repeat(total_width)).cyan().bold(),
        style("╝").cyan().bold(),
    );

    println!();
}

#[tokio::main]
async fn main() {
    print_startup_info();

    dotenvy::dotenv().ok();

    let binding = env::var("DISCORD_TOKEN").ok();
    let token = binding.as_deref();

    let token = match validate_token(token) {
        Ok(token) => token,
        Err(err) => panic!("{}", err.to_string()),
    };

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

    let framework = poise::Framework::builder()
        .setup(move |ctx, _, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .options(options)
        .build();

    let client = match serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("failed to create client: {e:?}");
            return;
        }
    };

    let state = BotState::new(client);

    let bot_state = state.clone();
    tokio::spawn(async move {
        bot_state.start().await;
    });
    web::run_web(state).await;
}
