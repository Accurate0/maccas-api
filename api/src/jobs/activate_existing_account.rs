use super::{error::JobError, Job, JobContext};
use crate::settings::McDonalds;
use base::{constants::mc_donalds, http::get_http_client};
use entity::accounts;
use libmaccas::{types::request::EmailRequest, ApiClient};
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QuerySelect};
use sensordata::{SensorDataRequest, SensorDataResponse};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ActivateExistingAccount {
    pub http_client: ClientWithMiddleware,
    pub mcdonalds_config: McDonalds,
    pub sensordata_api_base: String,
}

#[async_trait::async_trait]
impl Job for ActivateExistingAccount {
    fn name(&self) -> String {
        "activate_existing_account".to_owned()
    }

    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let mut client = ApiClient::new(
            mc_donalds::BASE_URL.to_string(),
            self.http_client.clone(),
            self.mcdonalds_config.client_id.clone(),
        );

        let response = client
            .security_auth_token(&self.mcdonalds_config.client_secret)
            .await?;
        client.set_login_token(&response.body.response.token);

        let account = accounts::Entity::find()
            .filter(accounts::Column::RefreshFailureCount.gt(3))
            .limit(1)
            .one(context.database)
            .await?;

        if account.is_none() {
            tracing::info!("no accounts in error");
            return Ok(());
        }

        let account = account.unwrap();

        let http_client = get_http_client()?;
        let sensor_data_response = http_client
            .get(format!(
                "{}/{}",
                self.sensordata_api_base,
                SensorDataRequest::path()
            ))
            .send()
            .await?
            .json::<SensorDataResponse>()
            .await?;

        let request = EmailRequest {
            device_id: account.device_id,
            registration_type: "traditional".to_owned(),
            customer_identifier: account.username.to_owned(),
        };

        let response = client
            .identity_email(&request, &sensor_data_response.sensor_data)
            .await?;

        tracing::info!("{:?}", response);
        tracing::info!(
            "[{}] attempted login for account with name",
            account.username,
        );

        Ok(())
    }
}
