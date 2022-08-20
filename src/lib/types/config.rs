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
        f.write_fmt(format_args!("email: {}", self.login_username))
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

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub client_id: String,
    pub client_secret: String,
    pub ignored_offer_ids: Vec<i64>,
    pub tables: Tables,
    pub sensor_data: String,
    pub api_key: String,
    pub users: Option<Vec<UserAccount>>,
    pub service_account: UserAccount,
    pub discord: DiscordConfig,
    pub log: LogConfig,
    pub protected_routes: ProtectedRouteConfig,
    pub refresh_counts: HashMap<String, i8>,
    pub image_bucket: String,
    pub admin_user_ids: Vec<String>,
}
