use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tables {
    pub token_cache: String,
    pub user_config: String,
    pub offer_cache: String,
    pub offer_cache_v2: String,
    pub offer_id: String,
    pub points: String,
    pub refresh_tracking: String,
    pub audit: String,
    pub audit_data: String,
    pub user_accounts: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Indexes {
    pub audit_user_id_index: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscordConfig {
    pub enabled: bool,
    pub webhooks: Vec<String>,
    pub avatar_url: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JwtConfig {
    pub validate: bool,
    pub jwks_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    pub enabled: bool,
    pub force_refresh: bool,
    pub bucket_name: String,
    pub copy_originals: bool,
    pub queue_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct McDonaldsConfig {
    pub client_id: String,
    pub client_secret: String,
    pub ignored_offer_ids: Vec<i64>,
    pub sensor_data: String,
    pub service_account_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseConfig {
    pub tables: Tables,
    pub indexes: Indexes,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub address: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshConfig {
    pub total_groups: HashMap<String, i8>,
    pub discord_error: DiscordConfig,
    pub enabled: bool,
    pub clear_deal_stacks: bool,
    pub enable_failure_handler: bool,
    pub failure_queue_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CleanupConfig {
    pub enabled: bool,
    pub queue_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub discord_deal_use: DiscordConfig,
    pub jwt: JwtConfig,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmailConfig {
    pub address: String,
    pub password: String,
    pub server_address: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountsConfig {
    pub email: EmailConfig,
    pub domain_name: String,
    pub enabled: bool,
    pub queue_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub mcdonalds: McDonaldsConfig,
    pub database: DatabaseConfig,
    pub refresh: RefreshConfig,
    pub api: ApiConfig,
    pub cleanup: CleanupConfig,
    pub images: ImageConfig,
    pub proxy: ProxyConfig,
    pub accounts: AccountsConfig,
}
