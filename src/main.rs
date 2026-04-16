use std::path::Path;

use console::style;
use skekbot_rs::{Config, consts, features::web};
use tracing::{info, warn};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

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
async fn main() -> anyhow::Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("warn,skekbot_rs=info"));

    let timer = fmt::time::ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string());

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().compact().with_target(true).with_timer(timer))
        .init();
    print_startup_info();

    let default_path = Path::new(consts::DATA_DIR).join("skekbot.toml");

    info!("looking for config in: {}", default_path.display());

    if !default_path.exists() {
        warn!("{} does not exist", default_path.display());

        if std::env::var("CREATE_CONFIG_FILE_IF_NOT_EXIST").unwrap_or_default() == "1" {
            warn!(
                "creating an empty config file at: {}",
                default_path.display()
            );

            if let Some(parent) = default_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&default_path, "")?;
        }
    }

    let config = Config::load(Some(&default_path))?;

    let _db = skekbot_rs::db::create_db().await?;

    let skekbot = skekbot_rs::create_skekbot(&config).await?;

    let bot_state_for_discord = skekbot.clone();

    tokio::spawn(async move {
        bot_state_for_discord.start().await;
    });

    web::run_web(skekbot).await;

    Ok(())
}
