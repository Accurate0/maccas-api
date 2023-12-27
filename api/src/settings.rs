use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::HashMap;

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
    pub downstream_health_checks: HashMap<String, String>,
    pub image_host_prefix: String,
    pub auth_secret: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("api.config.toml").required(true))
            .add_source(Environment::default().separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
