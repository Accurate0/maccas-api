use crate::{event_manager::EventManager, settings::Settings};
use api::Event;
use async_graphql::{InputObject, Object};
use base::constants::mc_donalds::OFFSET;
use entity::{accounts, points};
use opentelemetry::trace::TraceContextExt;
use sea_orm::{DatabaseConnection, EntityTrait, TransactionTrait, prelude::Uuid};
use std::time::Duration;

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

        let event_manager = context.data::<EventManager>()?;

        let trace_id = opentelemetry::Context::current()
            .span()
            .span_context()
            .trace_id()
            .to_string();

        if let Err(e) = event_manager
            .create_event(
                Event::RefreshPoints {
                    account_id: self.model.account_id,
                },
                Duration::from_secs(900),
                trace_id,
            )
            .await
        {
            tracing::error!("error creating event: {e}")
        };

        Ok(code_response.body.response.map(|r| r.random_code))
    }
}
