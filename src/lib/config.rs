use crate::{
    constants,
    types::config::{GeneralConfig, UserList},
};
use aws_sdk_s3::types::AggregatedBytes;
use config::Config;

impl GeneralConfig {
    async fn load_base_config_from_s3(
        client: &aws_sdk_s3::Client,
    ) -> Result<AggregatedBytes, anyhow::Error> {
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
}

impl UserList {
    pub async fn load_from_s3(
        shared_config: &aws_types::SdkConfig,
        region: &str,
        option: i8,
    ) -> Result<Self, anyhow::Error> {
        let s3_client = aws_sdk_s3::Client::new(shared_config);

        let resp = s3_client
            .get_object()
            .bucket(constants::CONFIG_BUCKET_NAME)
            .key(
                constants::config::REGION_ACCOUNTS_FILE
                    .replace("{region}", region)
                    .replace("{option}", &option.to_string()),
            )
            .send()
            .await?;
        let accounts_bytes = resp.body.collect().await?;

        let config = Config::builder().add_source(config::File::from_str(
            std::str::from_utf8(&accounts_bytes.into_bytes())?,
            config::FileFormat::Json,
        ));

        Ok(config.build()?.try_deserialize::<Self>()?)
    }
}
