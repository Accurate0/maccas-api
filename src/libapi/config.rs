use crate::constants;
use aws_sdk_s3::types::AggregatedBytes;
use config::Config;
use lambda_http::Error;

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
    pub cache_table_name_v2: String,
    pub offer_id_table_name: String,
    pub sensor_data: String,
    pub api_key: String,
    pub users: Vec<ApiConfigUsers>,
}

#[deprecated]
pub fn load(config: &str) -> ApiConfig {
    Config::builder()
        .add_source(config::File::from_str(config, config::FileFormat::Yaml))
        .build()
        .unwrap()
        .try_deserialize::<ApiConfig>()
        .expect("valid configuration present")
}

impl ApiConfig {
    async fn load_base_config_from_s3(client: &aws_sdk_s3::Client) -> Result<AggregatedBytes, Error> {
        let resp = client
            .get_object()
            .bucket(constants::CONFIG_BUCKET_NAME)
            .key(constants::BASE_CONFIG_FILE)
            .send()
            .await?;
        Ok(resp.body.collect().await?)
    }

    async fn load_sensor_data_from_s3(client: &aws_sdk_s3::Client) -> Result<AggregatedBytes, Error> {
        let resp = client
            .get_object()
            .bucket(constants::CONFIG_BUCKET_NAME)
            .key(constants::SENSOR_DATA_FILE)
            .send()
            .await?;
        Ok(resp.body.collect().await?)
    }

    async fn build_config_from_bytes(
        base_config: &AggregatedBytes,
        accounts: &AggregatedBytes,
        sensor_data: &AggregatedBytes,
    ) -> Result<Self, Error> {
        Ok(Config::builder()
            .add_source(config::File::from_str(
                std::str::from_utf8(&base_config.clone().into_bytes())?,
                config::FileFormat::Json,
            ))
            .add_source(config::File::from_str(
                std::str::from_utf8(&accounts.clone().into_bytes())?,
                config::FileFormat::Json,
            ))
            .add_source(config::File::from_str(
                std::str::from_utf8(&sensor_data.clone().into_bytes())?,
                config::FileFormat::Json,
            ))
            .build()?
            .try_deserialize::<Self>()?)
    }

    pub async fn load_from_s3(shared_config: &aws_types::SdkConfig) -> Result<Self, Error> {
        let s3_client = aws_sdk_s3::Client::new(&shared_config);
        let base_config_bytes = Self::load_base_config_from_s3(&s3_client).await?;
        let sensor_data_bytes = Self::load_sensor_data_from_s3(&s3_client).await?;

        let resp = s3_client
            .get_object()
            .bucket(constants::CONFIG_BUCKET_NAME)
            .key(constants::ALL_ACCOUNTS_FILE)
            .send()
            .await?;
        let all_accounts_bytes = resp.body.collect().await?;

        Ok(Self::build_config_from_bytes(&base_config_bytes, &all_accounts_bytes, &sensor_data_bytes).await?)
    }

    pub async fn load_from_s3_for_region(shared_config: &aws_types::SdkConfig, region: &String) -> Result<Self, Error> {
        let s3_client = aws_sdk_s3::Client::new(&shared_config);
        let base_config_bytes = Self::load_base_config_from_s3(&s3_client).await?;
        let sensor_data_bytes = Self::load_sensor_data_from_s3(&s3_client).await?;

        let resp = s3_client
            .get_object()
            .bucket(constants::CONFIG_BUCKET_NAME)
            .key(constants::REGION_ACCOUNTS_FILE.replace("{region}", region))
            .send()
            .await?;
        let accounts_bytes = resp.body.collect().await?;

        Ok(Self::build_config_from_bytes(&base_config_bytes, &accounts_bytes, &sensor_data_bytes).await?)
    }
}
