use crate::constants::config::{
    CONFIG_APIM_API_KEY_ID, CONFIG_APPLICATION_AUDIENCE_ID, CONFIG_JWT_BYPASS_ID,
};
use anyhow::Context;

#[async_trait]
pub trait SecretsManagerExtensions {
    async fn get_apim_api_key(&self) -> String;
    async fn get_jwt_bypass_key(&self) -> String;
    async fn get_application_id(&self) -> String;
}

#[async_trait]
impl SecretsManagerExtensions for aws_sdk_secretsmanager::Client {
    async fn get_apim_api_key(&self) -> String {
        self.get_secret_value()
            .secret_id(CONFIG_APIM_API_KEY_ID)
            .send()
            .await
            .context("must get APIM api key")
            .unwrap()
            .secret_string()
            .context("must get APIM api key")
            .unwrap()
            .to_string()
    }

    async fn get_jwt_bypass_key(&self) -> String {
        self.get_secret_value()
            .secret_id(CONFIG_JWT_BYPASS_ID)
            .send()
            .await
            .context("must get JWT bypass key")
            .unwrap()
            .secret_string()
            .context("must get JWT bypass key")
            .unwrap()
            .to_string()
    }

    async fn get_application_id(&self) -> String {
        self.get_secret_value()
            .secret_id(CONFIG_APPLICATION_AUDIENCE_ID)
            .send()
            .await
            .context("must get application id")
            .unwrap()
            .secret_string()
            .context("must get application id")
            .unwrap()
            .to_string()
    }
}
