use crate::constants;
use aws_config::{retry::RetryConfig, SdkConfig};

pub async fn get_shared_config() -> SdkConfig {
    aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .retry_config(RetryConfig::standard())
        .load()
        .await
}
