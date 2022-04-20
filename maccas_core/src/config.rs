#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfigUsers {
    pub account_name: String,
    pub login_username: String,
    pub login_password: String,
}

#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub client_id: String,
    pub client_secret: String,
    pub table_name: String,
    pub cache_table_name: String,
    pub users: Vec<ApiConfigUsers>,
}
