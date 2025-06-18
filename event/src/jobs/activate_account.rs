use std::{collections::HashSet, time::Duration};

use super::{error::JobError, Job, JobContext};
use crate::settings::{Email, McDonalds};
use anyhow::Context;
use base::constants::mc_donalds;
use base::http::get_http_client;
use entity::accounts;
use event::Event;
use libmaccas::{
    types::request::{ActivateAndSignInRequest, ActivationDevice, ClientInfo},
    ApiClient,
};
use mailparse::MailHeaderMap;
use opentelemetry::trace::TraceContextExt;
use regex::Regex;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use sensordata::{SensorDataRequest, SensorDataResponse};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug)]
pub struct ActivateAccountJob {
    pub http_client: ClientWithMiddleware,
    pub sensordata_api_base: String,
    pub mcdonalds_config: McDonalds,
    pub email_config: Email,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ActivateAccountContext {
    pub activated_accounts: HashSet<Uuid>,
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
        let imap_client = imap::ClientBuilder::new(&self.email_config.server_address, 993)
            .tls_kind(imap::TlsKind::Native)
            .connect()?;

        let mut imap_session = imap_client
            .login(
                self.email_config.address.clone(),
                self.email_config.password.clone(),
            )
            .map_err(|e| e.0)?;

        imap_session.select("INBOX")?;

        let http_client = get_http_client()?;

        let all_unseen_emails = imap_session.uid_search("(UNSEEN)")?;
        let re = Regex::new(r"ml=([a-zA-Z0-9]+)").unwrap();

        let mut activated_accounts = HashSet::new();

        for message_uid in all_unseen_emails.iter() {
            let messages = imap_session.uid_fetch(message_uid.to_string(), "RFC822")?;
            let message = messages.get(0).context("should have at least one")?;
            let body = message.body().expect("message did not have a body!");

            let parsed_email = mailparse::parse_mail(body)?;
            let body: &String = &parsed_email.get_body()?;
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
                    .one(context.database)
                    .await?
                    .ok_or(anyhow::Error::msg("no account found"))?;

                let device_id = &account.device_id;
                let magic_link = magic_link.unwrap().as_str().to_string();
                tracing::info!("code: {:?}", magic_link);
                tracing::info!("email to: {:?}", to.clone().as_str().to_string());

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

                let mut client = ApiClient::new(
                    mc_donalds::BASE_URL.to_string(),
                    self.http_client.clone(),
                    self.mcdonalds_config.client_id.clone(),
                );

                let security_auth_token_response = client
                    .security_auth_token(&self.mcdonalds_config.client_secret)
                    .await?;

                client.set_login_token(&security_auth_token_response.body.response.token);

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
                        &sensor_data_response.sensor_data,
                    )
                    .await?;

                if let Some(token_response) = response.body.response {
                    let account_id = account.id;
                    let mut active_model = account.into_active_model();
                    active_model.access_token = Set(token_response.access_token);
                    active_model.refresh_token = Set(token_response.refresh_token);
                    active_model.refresh_failure_count = Set(0);

                    active_model.update(context.database).await?;
                    activated_accounts.insert(account_id);
                }
            }
        }

        context
            .set(ActivateAccountContext { activated_accounts })
            .await?;

        Ok(())
    }

    async fn post_execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        if let Some(ActivateAccountContext { activated_accounts }) = context.get().await {
            let trace_id = opentelemetry::Context::current()
                .span()
                .span_context()
                .trace_id()
                .to_string();

            for activated_account in activated_accounts {
                let event = Event::RefreshAccount {
                    account_id: activated_account,
                };

                context
                    .event_manager
                    .create_event(event, Duration::from_secs(5), trace_id.clone())
                    .await?;
            }
        };
        Ok(())
    }
}
