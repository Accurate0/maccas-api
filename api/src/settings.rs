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

#[derive(Debug, Deserialize, Clone, Default)]
pub struct NewOffer {
    #[serde(default)]
    pub discord_urls: Vec<String>,
    #[serde(default)]
    pub external_urls: Vec<String>,
}

impl NewOffer {
    pub fn should_notify_discord(&self) -> bool {
        !self.discord_urls.is_empty()
    }

    pub fn should_notify_external(&self) -> bool {
        !self.external_urls.is_empty()
    }
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
    #[serde(default)]
    pub new_offer: NewOffer,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(
                Environment::default()
                    .separator("__")
                    .list_separator(",")
                    .with_list_parse_key("new_offer.discord_urls")
                    .with_list_parse_key("new_offer.external_urls")
                    .try_parsing(true),
            )
            .build()?;

        s.try_deserialize()
    }
}
