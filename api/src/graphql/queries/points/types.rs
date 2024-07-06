use std::time::Duration;

use crate::{graphql::ValidatedToken, settings::Settings};
use async_graphql::{InputObject, Object};
use base::constants::mc_donalds::OFFSET;
use entity::{accounts, points};
use event::{CreateEvent, CreateEventResponse, Event};
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{prelude::Uuid, DatabaseConnection, EntityTrait, TransactionTrait};

#[derive(InputObject)]
pub struct FilterInput {
    pub minimum_current_points: i64,
}

#[derive(InputObject)]
pub struct PointsByAccountIdInput {
    pub account_id: Uuid,
}

pub struct Points {
    pub model: points::Model,
    pub store_id: Option<String>,
}

#[Object]
impl Points {
    pub async fn account_id(&self) -> &Uuid {
        &self.model.account_id
    }

    pub async fn current_points(&self) -> &i64 {
        &self.model.current_points
    }

    pub async fn lifetime_points(&self) -> &i64 {
        &self.model.lifetime_points
    }

    pub async fn code(
        &self,
        context: &async_graphql::Context<'_>,
    ) -> async_graphql::Result<Option<String>> {
        let db = context.data::<DatabaseConnection>()?;
        let settings = context.data::<Settings>()?;
        let http_client = context.data::<ClientWithMiddleware>()?;

        if self.store_id.is_none() {
            return Err(async_graphql::Error::new(
                "must provide store id to get code",
            ));
        }

        let account_to_use = accounts::Entity::find_by_id(self.model.account_id)
            .one(db)
            .await?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        let proxy = reqwest::Proxy::all(settings.proxy.url.clone())?
            .basic_auth(&settings.proxy.username, &settings.proxy.password);

        let account_lock_txn = db.begin().await?;
        let api_client = base::maccas::get_activated_maccas_api_client(
            account_to_use,
            proxy,
            &settings.mcdonalds.client_id,
            &account_lock_txn,
        )
        .await?;
        account_lock_txn.commit().await?;

        let code_response = api_client
            .get_offers_dealstack(OFFSET, self.store_id.as_ref().unwrap())
            .await?;

        let cleanup_event = CreateEvent {
            event: Event::RefreshPoints {
                account_id: self.model.account_id,
            },
            delay: Duration::from_secs(900),
        };

        let request_url = format!("{}/{}", settings.event_api_base, CreateEvent::path());
        let request = http_client.post(request_url).json(&cleanup_event);
        let token = context.data_opt::<ValidatedToken>().map(|v| &v.0);

        let request = if let Some(token) = token {
            request.bearer_auth(token)
        } else {
            request
        };

        let response = request.send().await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED => {
                    let id = response.json::<CreateEventResponse>().await?.id;
                    tracing::info!("created refresh points event with id {}", id);
                }
                status => {
                    tracing::warn!("event failed with {} - {}", status, response.text().await?);
                }
            },
            Err(e) => tracing::warn!("event request failed with {}", e),
        }

        Ok(code_response.body.response.map(|r| r.random_code))
    }
}
