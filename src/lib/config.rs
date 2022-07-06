use std::fmt::Display;

use crate::constants;
use aws_sdk_s3::types::AggregatedBytes;
use config::Config;
use serde::{Deserialize, Serialize};

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
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiConfig {
    pub local_time_zone: String,
    pub client_id: String,
    pub client_secret: String,
    pub tables: Tables,
    pub sensor_data: String,
    pub api_key: String,
    pub users: Option<Vec<UserAccount>>,
    pub service_account: UserAccount,
}

impl ApiConfig {
    async fn load_base_config_from_s3(client: &aws_sdk_s3::Client) -> Result<AggregatedBytes, anyhow::Error> {
        let resp = client
            .get_object()
            .bucket(constants::CONFIG_BUCKET_NAME)
            .key(constants::config::BASE_FILE)
            .send()
            .await?;
        Ok(resp.body.collect().await?)
    }

    async fn build_config_from_bytes(
        base_config: &AggregatedBytes,
        accounts: Option<&AggregatedBytes>,
    ) -> Result<Self, anyhow::Error> {
        let config = Config::builder().add_source(config::File::from_str(
            std::str::from_utf8(&base_config.clone().into_bytes())?,
            config::FileFormat::Json,
        ));

        let config = if let Some(accounts) = accounts {
            config.add_source(config::File::from_str(
                std::str::from_utf8(&accounts.clone().into_bytes())?,
                config::FileFormat::Json,
            ))
        } else {
            config
        };

        Ok(config.build()?.try_deserialize::<Self>()?)
    }

    pub async fn load_from_s3(shared_config: &aws_types::SdkConfig) -> Result<Self, anyhow::Error> {
        let s3_client = aws_sdk_s3::Client::new(shared_config);
        let base_config_bytes = Self::load_base_config_from_s3(&s3_client).await?;

        Self::build_config_from_bytes(&base_config_bytes, None).await
    }

    pub async fn load_from_s3_with_region_accounts(
        shared_config: &aws_types::SdkConfig,
        region: &str,
    ) -> Result<Self, anyhow::Error> {
        let s3_client = aws_sdk_s3::Client::new(shared_config);
        let base_config_bytes = Self::load_base_config_from_s3(&s3_client).await?;

        let resp = s3_client
            .get_object()
            .bucket(constants::CONFIG_BUCKET_NAME)
            .key(constants::config::REGION_ACCOUNTS_FILE.replace("{region}", region))
            .send()
            .await?;
        let accounts_bytes = resp.body.collect().await?;

        Self::build_config_from_bytes(&base_config_bytes, Some(&accounts_bytes)).await
    }
}
