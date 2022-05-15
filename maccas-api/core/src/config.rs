use crate::constants;
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

pub async fn load_from_s3(shared_config: &aws_types::SdkConfig) -> ApiConfig {
    let s3_client = aws_sdk_s3::Client::new(&shared_config);
    let resp = s3_client
        .get_object()
        .bucket(constants::CONFIG_BUCKET_NAME)
        .key(constants::BASE_CONFIG_FILE)
        .send()
        .await
        .unwrap();
    let base_config_bytes = resp.body.collect().await.unwrap();

    let resp = s3_client
        .get_object()
        .bucket(constants::CONFIG_BUCKET_NAME)
        .key(constants::ALL_ACCOUNTS_FILE)
        .send()
        .await
        .unwrap();
    let all_accounts_bytes = resp.body.collect().await.unwrap();

    Config::builder()
        .add_source(config::File::from_str(
            std::str::from_utf8(&base_config_bytes.into_bytes()).unwrap(),
            config::FileFormat::Json,
        ))
        .add_source(config::File::from_str(
            std::str::from_utf8(&all_accounts_bytes.into_bytes()).unwrap(),
            config::FileFormat::Json,
        ))
        .build()
        .unwrap()
        .try_deserialize::<ApiConfig>()
        .expect("valid configuration present")
}

pub async fn load_from_s3_for_region(
    shared_config: &aws_types::SdkConfig,
    region: &String,
) -> ApiConfig {
    let s3_client = aws_sdk_s3::Client::new(&shared_config);
    let resp = s3_client
        .get_object()
        .bucket(constants::CONFIG_BUCKET_NAME)
        .key(constants::BASE_CONFIG_FILE)
        .send()
        .await
        .unwrap();
    let base_config_bytes = resp.body.collect().await.unwrap();

    let resp = s3_client
        .get_object()
        .bucket(constants::CONFIG_BUCKET_NAME)
        .key(constants::REGION_ACCOUNTS_FILE.replace("{region}", region))
        .send()
        .await
        .unwrap();
    let all_accounts_bytes = resp.body.collect().await.unwrap();

    Config::builder()
        .add_source(config::File::from_str(
            std::str::from_utf8(&base_config_bytes.into_bytes()).unwrap(),
            config::FileFormat::Json,
        ))
        .add_source(config::File::from_str(
            std::str::from_utf8(&all_accounts_bytes.into_bytes()).unwrap(),
            config::FileFormat::Json,
        ))
        .build()
        .unwrap()
        .try_deserialize::<ApiConfig>()
        .expect("valid configuration present")
}
