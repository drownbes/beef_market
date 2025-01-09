use std::path::Path;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub db: Db,
    pub ollama: OllamaConfig,
    pub geckodriver: GeckoDriver,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub embedding_model: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Db {
    pub conn_str: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct GeckoDriver {
    pub host: String,
    pub port: u16,
}

pub fn read_config(cfg_path: &Path) -> Result<AppConfig, config::ConfigError> {
    let cfg = config::Config::builder()
        .add_source(config::File::from(cfg_path))
        .build()?;

    cfg.try_deserialize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_config() {
        let _c = read_config(Path::new("config")).unwrap();
    }
}
