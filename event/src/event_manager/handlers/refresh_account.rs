use std::time::Duration;

use super::HandlerError;
use crate::{event_manager::EventManager, jobs::shared, settings::Settings};
use anyhow::Context;
use entity::accounts;
use opentelemetry::trace::TraceContextExt;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{EntityTrait, TransactionTrait};
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use uuid::Uuid;

#[instrument(skip(em))]
pub async fn refresh_account(account_id: Uuid, em: EventManager) -> Result<(), HandlerError> {
    tracing::info!("refresh account for {}", account_id);

    let settings = em.get_state::<Settings>();
    let http_client = em.get_state::<ClientWithMiddleware>();
    let db = em.db().begin().await?;

    let account = accounts::Entity::find_by_id(account_id)
        .one(&db)
        .await?
        .context("must find valid account")?;

    let events_to_dispatch = shared::refresh_account(
        account,
        http_client,
        &settings.mcdonalds,
        &db,
        CancellationToken::new(),
    )
    .await?;

    db.commit().await?;

    let trace_id = opentelemetry::Context::current()
        .span()
        .span_context()
        .trace_id()
        .to_string();

    for event in events_to_dispatch {
        em.create_event(event, Duration::from_secs(30), trace_id.clone())
            .await?;
    }

    Ok(())
}
