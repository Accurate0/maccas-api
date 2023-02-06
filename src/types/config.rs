use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserAccount {
    pub account_name: String,
    pub login_username: String,
    pub login_password: String,
}

impl Display for UserAccount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.login_username))
    }
}

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
    pub webp_quality: f32,
    pub queue_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct McDonaldsConfig {
    pub client_id: String,
    pub client_secret: String,
    pub ignored_offer_ids: Vec<i64>,
    pub sensor_data: String,
    pub service_account: UserAccount,
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
    pub refresh_counts: HashMap<String, i8>,
    pub discord_error: DiscordConfig,
    pub enabled: bool,
    pub clear_deal_stacks: bool,
    pub proxy: ProxyConfig,
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
    pub accounts: AccountsConfig,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserList {
    pub users: Vec<UserAccount>,
}
