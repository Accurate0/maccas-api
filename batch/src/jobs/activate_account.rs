use super::{error::JobError, Job, JobContext};
use crate::settings::{Email, McDonalds};
use anyhow::Context;
use base::constants::mc_donalds;
use entity::accounts;
use libmaccas::{
    types::request::{ActivateAndSignInRequest, ActivationDevice, ClientInfo},
    ApiClient,
};
use mailparse::MailHeaderMap;
use regex::Regex;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ActivateAccountJob {
    pub http_client: ClientWithMiddleware,
    pub mcdonalds_config: McDonalds,
    pub email_config: Email,
}

#[async_trait::async_trait]
impl Job for ActivateAccountJob {
    fn name(&self) -> String {
        "activate_account".to_owned()
    }

    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        // TODO: actually finish this, not needed yet apparently.. things kinda just work..
        let tls = native_tls::TlsConnector::builder().build().unwrap();
        let imap_client = imap::connect(
            (self.email_config.server_address.clone(), 993),
            self.email_config.server_address.clone(),
            &tls,
        )?;

        let mut imap_session = imap_client
            .login(
                self.email_config.address.clone(),
                self.email_config.password.clone(),
            )
            .map_err(|e| e.0)?;

        imap_session.select("INBOX")?;

        let mut client = ApiClient::new(
            mc_donalds::BASE_URL.to_string(),
            self.http_client.clone(),
            self.mcdonalds_config.client_id.clone(),
        );

        let response = client
            .security_auth_token(&self.mcdonalds_config.client_secret)
            .await?;
        client.set_login_token(&response.body.response.token);

        let all_unseen_emails = imap_session.uid_search("(UNSEEN)")?;
        for message_uid in all_unseen_emails.iter() {
            let messages = imap_session.uid_fetch(message_uid.to_string(), "RFC822")?;
            let message = messages.first().context("should have at least one")?;
            let body = message.body().expect("message did not have a body!");

            let parsed_email = mailparse::parse_mail(body)?;
            let body: &String = &parsed_email.get_body()?;
            let re = Regex::new(r"ml=([a-zA-Z0-9]+)").unwrap();
            let mut magic_link = None;
            for cap in re.captures_iter(body) {
                tracing::info!("capture: {:?}", cap);
                magic_link = cap.get(1);
            }

            let headers = parsed_email.get_headers();
            let to = headers
                .get_first_header("To")
                .context("must have to")?
                .get_value();
            let from = headers
                .get_first_header("From")
                .context("must have from")?
                .get_value();

            if !from.contains("accounts@au.mcdonalds.com") {
                tracing::warn!("skipping non maccas email, {:?}", from);
                continue;
            }

            if magic_link.is_some() {
                let account = accounts::Entity::find()
                    .filter(accounts::Column::Username.eq(to.clone()))
                    .one(&context.database)
                    .await?
                    .ok_or(anyhow::Error::msg("no account found"))?;

                let device_id = &account.device_id;
                let magic_link = magic_link.unwrap().as_str().to_string();
                tracing::info!("code: {:?}", magic_link);
                tracing::info!("email to: {:?}", to.clone().as_str().to_string());

                let response = client
                    .activate_and_signin(
                        &ActivateAndSignInRequest {
                            activation_link: magic_link,
                            client_info: ClientInfo {
                                device: ActivationDevice {
                                    device_unique_id: device_id.to_owned(),
                                    os: "android".to_owned(),
                                    os_version: "14".to_owned(),
                                },
                            },
                        },
                        &self.mcdonalds_config.sensor_data,
                    )
                    .await?;

                if let Some(token_response) = response.body.response {
                    let mut active_model = account.into_active_model();
                    active_model.access_token = Set(token_response.access_token);
                    active_model.refresh_token = Set(token_response.refresh_token);

                    active_model.update(&context.database).await?;
                }
            }
        }
        Ok(())
    }

    async fn cleanup(&self, _context: &JobContext) {}
}
