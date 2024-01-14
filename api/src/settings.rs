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
    pub event_api_base: String,
    pub auth_secret: String,
    pub places_api_key: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // TODO: envvar config path
        let s = Config::builder()
            .add_source(File::with_name("config/base.config.toml").required(false))
            .add_source(File::with_name("config/api.config.toml").required(true))
            .add_source(Environment::default().separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
