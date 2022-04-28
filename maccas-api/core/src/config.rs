use config::Config;

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
    pub offer_id_table_name: String,
    pub users: Vec<ApiConfigUsers>,
}

pub fn load(config: &str) -> ApiConfig {
    Config::builder()
        .add_source(config::File::from_str(config, config::FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize::<ApiConfig>()
        .expect("valid configuration present")
}
