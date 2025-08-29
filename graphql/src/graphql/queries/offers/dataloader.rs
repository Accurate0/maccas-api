use crate::{graphql::queries::offers::types::OfferCount, name_of, settings::Settings};
use async_graphql::dataloader::Loader;
use base::jwt::generate_internal_jwt;
use caching::OfferDetailsCache;
use chrono::DateTime;
use entity::{offer_details, offers};
use event::{CreateBulkEvents, CreateBulkEventsResponse, CreateEvent, Event};
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, JoinType, QueryFilter,
    QuerySelect, RelationTrait,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tracing::instrument;

pub struct OfferDetailsLoader {
    pub database: DatabaseConnection,
    pub http_client: ClientWithMiddleware,
    pub settings: Settings,
    pub cache: Option<OfferDetailsCache>,
}

impl OfferDetailsLoader {
    pub async fn trigger_cache_in_background(
        auth_secret: &str,
        event_api_base: &str,
        http_client: ClientWithMiddleware,
        proposition_ids: Vec<i64>,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("trigger cache for {proposition_ids:?}");
        let events = proposition_ids
            .iter()
            .map(|id| CreateEvent {
                event: Event::PopulateOfferDetailsCacheFor {
                    offer_proposition_id: *id,
                },
                delay: Duration::from_secs(0),
            })
            .collect();

        let request_url = format!("{}/{}", event_api_base, CreateBulkEvents::path());
        let token = generate_internal_jwt(auth_secret.as_ref(), "Maccas API", "Maccas Event")?;

        let request = http_client
            .post(request_url)
            .json(&CreateBulkEvents { events })
            .bearer_auth(token);

        let response = request.send().await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED => {
                    let ids = response.json::<CreateBulkEventsResponse>().await?.ids;
                    tracing::info!("created cache event with id {:?}", ids);
                }
                status => {
                    tracing::warn!("event failed with {} - {}", status, response.text().await?);
                }
            },
            Err(e) => tracing::warn!("event request failed with {}", e),
        }

        Ok(())
    }

    pub fn convert_from_cache_to_db(
        cached_value: caching::maccas::caching::OfferDetails,
        now: chrono::NaiveDateTime,
    ) -> offer_details::Model {
        offer_details::Model {
            proposition_id: cached_value.proposition_id,
            name: cached_value.name,
            description: cached_value.description,
            price: cached_value.price,
            short_name: cached_value.short_name,
            image_base_name: cached_value.image_base_name,
            created_at: if let Some(created_at) = cached_value.created_at {
                DateTime::from_timestamp(
                    created_at.seconds,
                    created_at.nanos.try_into().unwrap_or_default(),
                )
                .unwrap_or_default()
                .naive_utc()
            } else {
                now
            },
            updated_at: if let Some(updated_at) = cached_value.updated_at {
                DateTime::from_timestamp(
                    updated_at.seconds,
                    updated_at.nanos.try_into().unwrap_or_default(),
                )
                .unwrap_or_default()
                .naive_utc()
            } else {
                now
            },
            raw_data: None,
            categories: if cached_value.categories.is_empty() {
                None
            } else {
                Some(cached_value.categories)
            },
            migrated: cached_value.migrated,
        }
    }
}

impl Loader<i64> for OfferDetailsLoader {
    type Value = offer_details::Model;
    type Error = Arc<anyhow::Error>;

    #[instrument(name = "OfferDetailsLoader::load", skip(self, keys))]
    async fn load(&self, keys: &[i64]) -> Result<HashMap<i64, Self::Value>, Self::Error> {
        let cached_values = if let Some(ref cache) = self.cache {
            cache.get_all(keys).await.map_err(anyhow::Error::from)?
        } else {
            vec![]
        };

        let mut check_db_for = vec![];
        let mut cache_values = HashMap::new();
        let now = chrono::offset::Utc::now().naive_utc();

        for (id, value) in keys.iter().zip(cached_values) {
            match value {
                Some(v) => {
                    cache_values.insert(*id, Self::convert_from_cache_to_db(v, now));
                }
                None => {
                    check_db_for.push(*id);
                }
            }
        }

        tracing::Span::current()
            .record("cached", cache_values.len())
            .record("db", check_db_for.len());

        tracing::info!(
            "cached count: {}, checking db count: {}",
            cache_values.len(),
            check_db_for.len()
        );

        if !check_db_for.is_empty() {
            let check_db_for_background = check_db_for.clone();
            let event_api_base = self.settings.event_api_base.clone();
            let auth_secret = self.settings.auth_secret.clone();
            let http_client = self.http_client.clone();

            tokio::spawn(async move {
                Self::trigger_cache_in_background(
                    &auth_secret,
                    &event_api_base,
                    http_client,
                    check_db_for_background,
                )
                .await
                .inspect_err(|e| tracing::error!("error refreshing cache: {e}"))
            });

            let db_values = offer_details::Entity::find()
                .filter(offer_details::Column::PropositionId.is_in(check_db_for))
                .all(&self.database)
                .await
                .map_err(anyhow::Error::from)?
                .into_iter()
                .map(|o| (o.proposition_id, o))
                .collect::<HashMap<_, _>>();

            cache_values.extend(db_values);
        }

        Ok(cache_values)
    }
}

pub struct OfferCountDataLoader {
    pub database: DatabaseConnection,
}

impl Loader<String> for OfferCountDataLoader {
    type Value = i64;
    type Error = Arc<DbErr>;

    #[instrument(name = "OfferCountDataLoader::load", skip(self, names))]
    async fn load(&self, names: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let db = &self.database;
        let all_locked_accounts = entity::account_lock::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|a| a.id);

        let mut conditions = Condition::all();
        for locked_account in all_locked_accounts {
            conditions = conditions.add(offers::Column::AccountId.ne(locked_account));
        }

        let now = chrono::offset::Utc::now().naive_utc();

        let conditions = conditions
            .add(offers::Column::ValidTo.gt(now))
            .add(offers::Column::ValidFrom.lt(now))
            .add(offer_details::Column::ShortName.is_in(names));

        Ok(offers::Entity::find()
            .select_only()
            .filter(conditions.clone())
            .join(JoinType::InnerJoin, offers::Relation::OfferDetails.def())
            .column(offer_details::Column::ShortName)
            .column_as(
                offer_details::Column::ShortName.count(),
                name_of!(count in OfferCount),
            )
            .group_by(offer_details::Column::ShortName)
            .into_model::<OfferCount>()
            .all(db)
            .await?
            .into_iter()
            .map(|o| (o.short_name, o.count))
            .collect::<HashMap<_, _>>())
    }
}
