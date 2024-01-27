use config::{Config, ConfigError, Environment, File};
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
    pub client_secret: String,
    pub sensor_data: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: Database,
    pub proxy: Proxy,
    pub mcdonalds: McDonalds,
    pub auth_secret: String,
    pub email_domain_name: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/base.config.toml").required(false))
            .add_source(File::with_name("config/batch.config.toml").required(false))
            .add_source(Environment::default().separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
