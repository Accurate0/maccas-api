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
pub struct LogConfig {
    pub ignored_user_ids: Vec<String>,
    pub local_time_zone: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProtectedRouteConfig {
    pub allowed_user_ids: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JwtConfig {
    pub validate: bool,
    pub bypass_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    pub force_refresh: bool,
    pub bucket_name: String,
    pub copy_originals: bool,
    pub webp_quality: f32,
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
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServiceConfig {
    pub images: ImageConfig,
    pub refresh_counts: HashMap<String, i8>,
    pub discord_refresh_error: DiscordConfig,
    pub refresh_images: bool,
    pub refresh_offer_cache: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub api_key: String,
    pub discord_deal_use: DiscordConfig,
    pub log: LogConfig,
    pub protected_routes: ProtectedRouteConfig,
    pub admin_user_ids: Vec<String>,
    pub jwt: JwtConfig,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GeneralConfig {
    pub mcdonalds: McDonaldsConfig,
    pub database: DatabaseConfig,
    pub service: ServiceConfig,
    pub api: ApiConfig,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserList {
    pub users: Vec<UserAccount>,
}