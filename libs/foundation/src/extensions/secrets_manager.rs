use anyhow::Context;
use async_trait::async_trait;

#[async_trait]
pub trait SecretsManagerExtensions {
    async fn get_secret(&self, id: &str) -> Result<String, anyhow::Error>;
}

#[async_trait]
impl SecretsManagerExtensions for aws_sdk_secretsmanager::Client {
    async fn get_secret(&self, id: &str) -> Result<String, anyhow::Error> {
        Ok(self
            .get_secret_value()
            .secret_id(id)
            .send()
            .await
            .context("must get secret value")?
            .secret_string()
            .context("must get secret value")?
            .to_string())
    }
}
