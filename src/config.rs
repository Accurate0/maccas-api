use crate::{
    constants::{
        self,
        config::{BASE_FILE, CONFIG_BUCKET_NAME},
    },
    types::config::{GeneralConfig, UserList},
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

impl UserList {
    pub async fn load_from_s3(
        shared_config: &aws_types::SdkConfig,
        region: &str,
        option: i8,
    ) -> Result<Self, anyhow::Error> {
        let s3_client = aws_sdk_s3::Client::new(shared_config);
        let user_list_file = constants::config::REGION_ACCOUNTS_FILE
            .replace("{region}", region)
            .replace("{option}", &option.to_string());

        foundation::config::load_config_from_s3(
            &s3_client,
            CONFIG_BUCKET_NAME,
            user_list_file,
            config::FileFormat::Json,
        )
        .await
    }
}
