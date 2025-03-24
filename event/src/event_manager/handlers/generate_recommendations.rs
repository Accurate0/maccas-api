use std::collections::HashMap;

use super::HandlerError;
use crate::event_manager::EventManager;
use chrono::{Days, NaiveDateTime};
use entity::{offer_audit, offer_details, sea_orm_active_enums::Action};
use itertools::Itertools;
use sea_orm::{sea_query::OnConflict, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use tokio::runtime::Handle;
use tracing::instrument;
use uuid::Uuid;

pub const RECENCY_LAST_X_DAYS_WEIGHT: f64 = 2.5;
pub const LAST_X_DAYS: u64 = 30;
pub const RECOMMENDATION_COUNT: usize = 5;

#[instrument(skip(em))]
pub async fn generate_recommendations(user_id: Uuid, em: EventManager) -> Result<(), HandlerError> {
    let db = em.db();
    // We're gonna do something insane

    let bulk_data = offer_audit::Entity::find()
        .filter(offer_audit::Column::UserId.eq(user_id))
        .filter(offer_audit::Column::Action.eq(Action::Add))
        .left_join(offer_details::Entity)
        .all(db)
        .await?;

    let rt = Handle::current();
    let model: entity::recommendations::ActiveModel = rt
        .spawn_blocking(move || {
            let x_days_ago = chrono::offset::Utc::now()
                .naive_utc()
                .checked_sub_days(Days::new(LAST_X_DAYS))
                .unwrap();

            let recommended_offer_proposition_ids = bulk_data
                .into_iter()
                .into_group_map_by(|m| m.proposition_id)
                .into_iter()
                .map(|(k, v)| (k, v.into_iter().map(|m| m.created_at).collect()))
                .collect::<HashMap<i64, Vec<NaiveDateTime>>>()
                .into_iter()
                .map(|(id, recency)| {
                    let total = recency.len();
                    let usages_in_last_x_days = recency
                        .into_iter()
                        .filter(|d| *d > x_days_ago)
                        .collect_vec()
                        .len();

                    let score: f64 = (usages_in_last_x_days as f64 * RECENCY_LAST_X_DAYS_WEIGHT)
                        + (total - usages_in_last_x_days) as f64;

                    (id, score as usize)
                })
                .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
                .take(RECOMMENDATION_COUNT)
                .map(|r| r.0)
                .collect_vec();

            entity::recommendations::ActiveModel {
                user_id: Set(user_id),
                offer_proposition_ids: Set(recommended_offer_proposition_ids),
                ..Default::default()
            }
        })
        .await?;

    entity::recommendations::Entity::insert(model)
        .on_conflict(
            OnConflict::column(entity::recommendations::Column::UserId)
                .update_column(entity::recommendations::Column::OfferPropositionIds)
                .to_owned(),
        )
        .exec_without_returning(db)
        .await?;

    Ok(())
}
