use std::process::exit;

use once_cell::sync::Lazy;
use validator::Validate;

#[derive(serde::Deserialize, Clone)]
pub struct AppConfig {
    pub db: Db,
    pub ollama: OllamaConfig,
    pub geckodriver: GeckoDriver,
}

#[derive(serde::Deserialize, Clone)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub embedding_model: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct Db {
    path: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct GeckoDriver {
    host: String,
    port: u16,
}

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| {
    read_config().unwrap_or_else(|e| {
        println!("Error loading config:\n  {e:?}\n");
        exit(12)
    })
});

pub fn read_config() -> Result<AppConfig, config::ConfigError> {
    let cfg = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    cfg.try_deserialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config() {
        let _c = read_config().unwrap();
    }
}
