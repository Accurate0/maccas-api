use crate::{
    constants::config::{BASE_FILE, CONFIG_BUCKET_NAME},
    types::config::GeneralConfig,
};

impl GeneralConfig {
    pub async fn load_from_s3(shared_config: &aws_types::SdkConfig) -> Result<Self, anyhow::Error> {
        let s3_client = aws_sdk_s3::Client::new(shared_config);
        foundation::config::load_config_from_s3(
            &s3_client,
            CONFIG_BUCKET_NAME,
            BASE_FILE,
            config::FileFormat::Json,
        )
        .await
    }
}
