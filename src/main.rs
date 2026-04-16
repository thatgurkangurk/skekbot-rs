use std::{env, path::Path};

use console::style;
use skekbot_rs::{Config, consts, features::web};

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
    print_startup_info();

    dotenvy::dotenv().ok();

    let default_path = Path::new(consts::DATA_DIR).join("skekbot.toml");

    

    println!(
        "{} {}",
        style("➜").cyan().bold(),
        style(format!("looking for config in: {}", default_path.display())).dim()
    );

    if !default_path.exists() {
        println!(
            "{} {}",
            style("⚠").yellow().bold(),
            style(format!("warning: {} does not exist!", default_path.display())).yellow()
        );

        if std::env::var("CREATE_CONFIG_FILE_IF_NOT_EXIST").unwrap_or_default() == "1" {
            println!(
                "{} {}",
                style("📝").cyan().bold(),
                style(format!("creating empty config file at: {}", default_path.display())).cyan()
            );

            if let Some(parent) = default_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&default_path, "")?;
        }
    }

    let config = Config::load(Some(&default_path))?;

    let skekbot = skekbot_rs::create_skekbot(&config).await?;

    let bot_state_for_discord = skekbot.clone();

    tokio::spawn(async move {
        bot_state_for_discord.start().await;
    });

    web::run_web(skekbot).await;

    Ok(())
}
