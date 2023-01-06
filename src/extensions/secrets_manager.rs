use anyhow::Context;

use crate::constants::config::{CONFIG_APIM_API_KEY_ID, CONFIG_JWT_BYPASS_ID};

#[async_trait]
pub trait SecretsManagerExtensions {
    async fn get_apim_api_key(&self) -> &str;
    async fn get_jwt_bypass_key(&self) -> &str;
}

#[async_trait]
impl SecretsManagerExtensions for aws_sdk_secretsmanager::Client {
    async fn get_apim_api_key(&self) -> &str {
        self.get_secret_value()
            .secret_id(CONFIG_APIM_API_KEY_ID)
            .send()
            .await
            .context("must get APIM api key")
            .unwrap()
            .secret_string()
            .context("must get APIM api key")
            .unwrap()
    }

    async fn get_jwt_bypass_key(&self) -> &str {
        self.get_secret_value()
            .secret_id(CONFIG_JWT_BYPASS_ID)
            .send()
            .await
            .context("must get JWT bypass key")
            .unwrap()
            .secret_string()
            .context("must get JWT bypass key")
            .unwrap()
    }
}
