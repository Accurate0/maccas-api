use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tables {
    pub token_cache: String,
    pub user_config: String,
    pub account_cache: String,
    pub deal_cache: String,
    pub locked_offers: String,
    pub points: String,
    pub refresh_tracking: String,
    pub audit: String,
    pub last_refresh: String,
    pub user_accounts: String,
    pub users: String,
    pub user_tokens: String,
    pub current_deals: String,
    pub registration_tokens: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Indexes {
    pub audit_user_id_index: String,
    pub current_deals_offer_proposition_id: String,
    pub current_deals_account_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JwtConfig {
    pub application_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    pub enabled: bool,
    pub force_refresh: bool,
    pub bucket_name: String,
    pub copy_originals: bool,
    pub webp_quality: f32,
    pub queue_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct McDonaldsConfig {
    pub client_id: String,
    pub client_secret: String,
    pub ignored_offer_ids: Vec<i64>,
    pub sensor_data: String,
    pub service_account_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseConfig {
    pub tables: Tables,
    pub indexes: Indexes,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub address: String,
    pub username: String,
    pub password: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefreshConfig {
    pub total_groups: HashMap<String, i8>,
    pub enabled: bool,
    pub clear_deal_stacks: bool,
    pub enable_failure_handler: bool,
    pub failure_queue_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CleanupConfig {
    pub enabled: bool,
    pub queue_name: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub jwt: JwtConfig,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct EmailConfig {
    pub address: String,
    pub password: String,
    pub server_address: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccountsConfig {
    pub email: EmailConfig,
    pub domain_name: String,
    pub enabled: bool,
    pub queue_name: String,
}

#[derive(Clone, Deserialize, Debug)]
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
