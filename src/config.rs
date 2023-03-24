use crate::{
    constants::config::{BASE_FILE, CONFIG_BUCKET_NAME},
    types::config::GeneralConfig,
};
use config::Config;
use foundation::config::sources::s3::S3Source;

impl GeneralConfig {
    pub async fn load(shared_config: &aws_types::SdkConfig) -> Result<Self, anyhow::Error> {
        let s3_client = aws_sdk_s3::Client::new(shared_config);
        let s3_source = S3Source::new(
            CONFIG_BUCKET_NAME,
            BASE_FILE,
            config::FileFormat::Json,
            s3_client,
        );

        Config::builder()
            .add_async_source(s3_source)
            .build()
            .await?
            .try_deserialize::<Self>()
            .map_err(anyhow::Error::new)
    }
}
