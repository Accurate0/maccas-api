use super::HandlerError;
use crate::{event_manager::EventManager, settings::Settings};
use anyhow::Context;
use entity::{accounts, points};
use sea_orm::{sea_query::OnConflict, EntityTrait, IntoActiveModel, TransactionTrait};
use tracing::instrument;
use uuid::Uuid;

#[instrument(skip(em))]
pub async fn refresh_points(account_id: Uuid, em: EventManager) -> Result<(), HandlerError> {
    tracing::info!("refresh points for {}", account_id);

    let settings = em.get_state::<Settings>();
    let db = em.db();

    let account = accounts::Entity::find_by_id(account_id)
        .one(db)
        .await?
        .context("Must find valid account")?;

    let proxy = reqwest::Proxy::all(settings.proxy.url.clone())?
        .basic_auth(&settings.proxy.username, &settings.proxy.password);

    let account_lock_txn = db.begin().await?;
    let api_client = base::maccas::get_activated_maccas_api_client(
        account,
        proxy,
        &settings.mcdonalds.client_id,
        &account_lock_txn,
    )
    .await?;
    account_lock_txn.commit().await?;

    let txn = db.begin().await?;

    let points = api_client.get_customer_points().await?;
    let points_model =
        converters::Database::convert_points_response(&points.body.response, account_id)?
            .0
            .into_active_model();

    points::Entity::insert(points_model)
        .on_conflict(
            OnConflict::column(points::Column::AccountId)
                .update_columns([
                    points::Column::LifetimePoints,
                    points::Column::CurrentPoints,
                ])
                .to_owned(),
        )
        .exec(&txn)
        .await?;

    txn.commit().await?;

    Ok(())
}
