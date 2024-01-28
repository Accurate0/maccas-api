use super::{error::JobError, Job, JobContext};
use crate::settings::{Email, McDonalds};
use base::constants::mc_donalds;
use entity::accounts;
use libmaccas::{
    types::request::{
        AcceptancePolicies, Address, Audit, Credentials, Device, Policies, Preference,
        RegistrationRequest, Subscription,
    },
    ApiClient,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::StdRng,
    SeedableRng,
};
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{prelude::Uuid, ActiveModelTrait, Set};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct CreateAccountJob {
    pub http_client: ClientWithMiddleware,
    pub mcdonalds_config: McDonalds,
    pub email_config: Email,
}

#[async_trait::async_trait]
impl Job for CreateAccountJob {
    fn name(&self) -> String {
        "create_account".to_owned()
    }

    // TODO: needs refreshed at datetime as well, since updated at is updated by updating tokens alone
    // that can happen at any point really
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

        let mut rng = StdRng::from_entropy();

        let first_name = "Lachlan".to_owned();
        let last_name = "Wells".to_owned();

        let device_id = Alphanumeric.sample_string(&mut rng, 16);
        let username_prefix = Alphanumeric.sample_string(&mut rng, 24);
        let username = format!("{}@{}", username_prefix, self.email_config.domain_name);

        let request = RegistrationRequest {
            address: Address {
                country: "AU".to_string(),
                zip_code: "6233".to_string(),
            },
            audit: Audit {
                registration_channel: "M".to_string(),
            },
            credentials: Credentials {
                login_username: username.to_string(),
                password: None,
                send_magic_link: Some(true),
                type_field: "email".to_string(),
            },
            device: Device {
                device_id: device_id.to_string(),
                device_id_type: "AndroidId".to_string(),
                is_active: "Y".to_string(),
                os: "android".to_string(),
                os_version: "14".to_string(),
                timezone: "Australia/Perth".to_string(),
            },
            email_address: username.to_string(),
            first_name: first_name.clone(),
            last_name: last_name.clone(),
            opt_in_for_marketing: false,
            policies: Policies {
                acceptance_policies: AcceptancePolicies { n1: true, n4: true },
            },
            preferences: serde_json::from_str::<Vec<Preference>>(include_str!(
                "./resources/preferences.json"
            ))
            .unwrap(),
            subscriptions: serde_json::from_str::<Vec<Subscription>>(include_str!(
                "./resources/subscriptions.json"
            ))
            .unwrap(),
        };

        let response = client
            .customer_registration(&request, &self.mcdonalds_config.sensor_data)
            .await?;

        tracing::info!(
            "[{}] created account with name {} {}",
            request.email_address,
            first_name,
            last_name
        );

        accounts::ActiveModel {
            id: Set(Uuid::new_v4()),
            username: Set(username),
            password: Set(None),
            access_token: Set(response.body.response.access_token),
            refresh_token: Set(response.body.response.refresh_token),
            device_id: Set(device_id),
            ..Default::default()
        }
        .insert(&context.database)
        .await?;

        Ok(())
    }

    async fn cleanup(&self, _context: &JobContext) {}
}
