use validator::Validate;

#[derive(serde::Deserialize, Clone)]
pub struct AppConfig {
    pub db: Db,
    pub ollama: OllamaConfig,
    pub rimi: ShopConfig,
    pub selver: ShopConfig,
    pub barbora: ShopConfig,
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

#[derive(serde::Deserialize, Clone, Validate)]
pub struct ShopConfig {
    #[validate(url)]
    base_url: String,
    #[validate(url)]
    path_to_beef: String,
}

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
