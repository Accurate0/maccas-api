use anyhow::Context;

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

    pub async fn load_all_from_s3(
        shared_config: &aws_types::SdkConfig,
    ) -> Result<Self, anyhow::Error> {
        let s3_client = aws_sdk_s3::Client::new(shared_config);
        let object_list = s3_client
            .list_objects()
            .bucket(CONFIG_BUCKET_NAME)
            .send()
            .await?;
        let object_list = object_list
            .contents()
            .context("bucket should have contents")
            .unwrap();

        let mut total_user_list = Vec::new();

        for object in object_list {
            match object.key() {
                Some(key) if key.contains("accounts") => {
                    let mut user_list: UserList = foundation::config::load_config_from_s3(
                        &s3_client,
                        CONFIG_BUCKET_NAME,
                        key,
                        config::FileFormat::Json,
                    )
                    .await?;

                    total_user_list.append(&mut user_list.users);
                }
                Some(_) => continue,
                None => continue,
            }
        }

        Ok(UserList {
            users: total_user_list,
        })
    }
}
