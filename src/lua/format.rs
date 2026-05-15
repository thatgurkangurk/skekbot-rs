use std::path::Path;

use stylua_lib::{Config as StyluaConfig, OutputVerification};

fn get_stylua_config() -> StyluaConfig {
    let config_path = Path::new(".stylua.toml");

    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(config_path)
            && let Ok(config) = toml::from_str(&content)
        {
            return config;
        }
        tracing::warn!("found .stylua.toml but failed to parse it. using defaults.");
    }

    StyluaConfig::default()
}

pub fn format_code(code: &str) -> anyhow::Result<String> {
    let config = get_stylua_config();

    let formatted_content = stylua_lib::format_code(code, config, None, OutputVerification::None)
        .map_err(|e| anyhow::anyhow!("stylua error: {e}"))?;

    Ok(formatted_content)
}
