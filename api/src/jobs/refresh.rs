use super::{Job, JobContext, error::JobError, shared};
use crate::OfferDetailsCache;
use anyhow::Context as _;
use api::Event;
use base::constants::MACCAS_ACCOUNT_REFRESH_FAILURE;
use entity::accounts;
use opentelemetry::trace::TraceContextExt;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{
    ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
    sea_query::{LockBehavior, LockType},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct RefreshJob {
    pub http_client: ClientWithMiddleware,
    pub mcdonalds_config: crate::settings::McDonalds,
}

#[derive(Serialize, Deserialize)]
struct RefreshContext {
    events_to_dispatch: Vec<Event>,
}

#[async_trait::async_trait]
impl Job for RefreshJob {
    fn name(&self) -> String {
        "refresh".to_owned()
    }

    // TODO: needs refreshed at datetime as well, since updated at is updated by updating tokens alone
    // that can happen at any point really
    async fn execute(
        &self,
        context: &JobContext,
        cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let account_to_refresh = accounts::Entity::find()
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
            .filter(accounts::Column::Active.eq(true))
            .filter(accounts::Column::RefreshFailureCount.lte(MACCAS_ACCOUNT_REFRESH_FAILURE))
            .order_by_asc(accounts::Column::OffersRefreshedAt)
            .one(context.database_connection)
            .await?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        let caching = context.event_manager.try_get_state::<OfferDetailsCache>();
        let events_to_dispatch = shared::refresh_account(
            account_to_refresh,
            &self.http_client,
            &self.mcdonalds_config,
            context.database,
            context.database_connection,
            caching,
            cancellation_token,
        )
        .await?;

        context.set(RefreshContext { events_to_dispatch }).await?;

        Ok(())
    }

    async fn post_execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let refresh_context = context
            .get::<RefreshContext>()
            .await
            .context("must have a context")?;

        if refresh_context.events_to_dispatch.is_empty() {
            tracing::info!("no events to dispatch");
            return Ok(());
        }

        let trace_id = opentelemetry::Context::current()
            .span()
            .span_context()
            .trace_id()
            .to_string();

        for event in refresh_context.events_to_dispatch {
            context
                .event_manager
                .create_event(event, Duration::from_secs(30), trace_id.clone())
                .await?;
        }

        Ok(())
    }
}
