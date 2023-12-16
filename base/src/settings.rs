use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Proxy {
    pub username: String,
    pub password: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct McDonalds {
    pub client_id: String,
    pub client_secret: String,
    pub sensor_data: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub proxy: Proxy,
    pub mcdonalds: McDonalds,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/local.toml").required(false))
            .add_source(Environment::default().separator("__"))
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        s.try_deserialize()
    }
}
