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
    pub client_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Email {
    pub address: String,
    pub password: String,
    pub server_address: String,
    pub domain_name: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: Database,
    pub disable_jobs: Vec<String>,
    pub proxy: Proxy,
    pub mcdonalds: McDonalds,
    pub auth_secret: String,
    pub email: Email,
    pub openai_api_key: String,
    pub sensordata_api_base: String,
    pub event_api_base: String,
    pub recommendations_api_base: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(
                Environment::default()
                    .separator("__")
                    .list_separator(",")
                    .with_list_parse_key("disable_jobs")
                    .try_parsing(true),
            )
            .build()?;

        s.try_deserialize()
    }
}
