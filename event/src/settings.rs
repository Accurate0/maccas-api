use config::{Config, ConfigError, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Proxy {
    pub username: String,
    pub password: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McDonalds {
    pub client_id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImagesBucket {
    pub access_key_id: String,
    pub access_secret_key: String,
    pub endpoint: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: Database,
    pub proxy: Proxy,
    pub auth_secret: String,
    pub mcdonalds: McDonalds,
    pub images_bucket: ImagesBucket,
    pub recommendations_api_base: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(Environment::default().separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
