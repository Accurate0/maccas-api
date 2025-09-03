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
pub struct ImagesBucket {
    pub access_key_id: String,
    pub access_secret_key: String,
    pub endpoint: String,
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
    pub proxy: Proxy,
    pub auth_secret: String,
    pub mcdonalds: McDonalds,
    pub images_bucket: ImagesBucket,
    pub email: Email,
    pub openai_api_key: String,
    pub sensordata_api_base: String,
    pub recommendations_api_base: String,
    pub external_webhook_secret: String,
    pub places_api_key: String,
    pub redis_connection_string: Option<String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(Environment::default().separator("__"))
            .build()?;

        s.try_deserialize()
    }
}
