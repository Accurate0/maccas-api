use super::{error::JobError, Job, JobContext, JobType};
use crate::settings::McDonalds;
use anyhow::Context as _;
use base::{constants::mc_donalds, http::get_http_client, jwt::generate_internal_jwt};
use converters::Database;
use entity::{account_lock, accounts, offer_details, offer_history, offers};
use event::{CreateBulkEvents, CreateBulkEventsResponse, CreateEvent, Event};
use libmaccas::ApiClient;
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{
    sea_query::{LockBehavior, LockType, OnConflict},
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait, TryIntoModel,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct RefreshJob {
    pub event_api_base: String,
    pub http_client: ClientWithMiddleware,
    pub auth_secret: String,
    pub mcdonalds_config: McDonalds,
}

#[derive(Serialize, Deserialize)]
struct RefreshContext {
    events_to_dispatch: Vec<CreateEvent>,
}

impl RefreshJob {
    async fn create_bulk_events(
        &self,
        http_client: &ClientWithMiddleware,
        token: &str,
        events: Vec<CreateEvent>,
    ) -> Result<(), JobError> {
        let request_url = format!(
            "{}/{}",
            self.event_api_base,
            event::CreateBulkEvents::path()
        );

        let request = http_client
            .post(&request_url)
            .json(&CreateBulkEvents { events })
            .bearer_auth(token);

        let response = request.send().await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED => {
                    let id = response.json::<CreateBulkEventsResponse>().await?.ids;
                    tracing::info!("created events with id {:?}", id);
                }
                status => {
                    tracing::warn!("event failed with {} - {}", status, response.text().await?);
                }
            },
            Err(e) => tracing::warn!("event request failed with {}", e),
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Job for RefreshJob {
    fn name(&self) -> String {
        "refresh".to_owned()
    }

    fn job_type(&self) -> JobType {
        JobType::Schedule("0 */3 * * * *".parse().unwrap())
    }

    // TODO: needs refreshed at datetime as well, since updated at is updated by updating tokens alone
    // that can happen at any point really
    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let account_to_refresh = accounts::Entity::find()
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
            .filter(accounts::Column::Active.eq(true))
            .filter(accounts::Column::RefreshFailureCount.lte(3))
            .order_by_asc(accounts::Column::OffersRefreshedAt)
            .one(context.database)
            .await?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        let account_id = account_to_refresh.id.to_owned();
        let current_failure_count = account_to_refresh.refresh_failure_count;
        let api_client = async {
            tracing::info!("refreshing account: {:?}", &account_id);

            let mut api_client = libmaccas::ApiClient::new(
                base::constants::mc_donalds::BASE_URL.to_owned(),
                self.http_client.clone(),
                self.mcdonalds_config.client_id.clone(),
            );

            api_client.set_auth_token(&account_to_refresh.access_token);
            let response = api_client
                .customer_login_refresh(&account_to_refresh.refresh_token)
                .await?;

            let response = response
                .body
                .response
                .ok_or_else(|| anyhow::Error::msg("access token refresh failed"))?;

            api_client.set_auth_token(&response.access_token);

            let mut update_model = account_to_refresh.into_active_model();

            update_model.access_token = Set(response.access_token);
            update_model.refresh_token = Set(response.refresh_token);
            tracing::info!("new tokens fetched, updating database");

            update_model.update(context.database).await?;

            Ok::<ApiClient, anyhow::Error>(api_client)
        }
        .await;

        let api_client = match api_client {
            Ok(api_client) => api_client,
            Err(e) => {
                tracing::warn!("increasing error count: {}", current_failure_count + 1);
                accounts::Entity::update(accounts::ActiveModel {
                    id: sea_orm::Unchanged(account_id),
                    refresh_failure_count: sea_orm::Set(current_failure_count + 1),
                    ..Default::default()
                })
                .exec(context.database)
                .await?;

                return Err(e.into());
            }
        };

        let offers = api_client
            .get_offers(
                mc_donalds::DISTANCE,
                mc_donalds::LATITUDE,
                mc_donalds::LONGITUDE,
                "",
                mc_donalds::OFFSET,
            )
            .await?;

        let offer_list = offers.body.response.unwrap_or_default();
        tracing::info!("{} offers found", offer_list.offers.len());

        let mut added_offers = vec![];

        let proposition_ids = offer_list.offers.iter().map(|o| o.offer_proposition_id);

        for id in proposition_ids {
            let api_client_cloned = api_client.clone();

            // TODO: bad
            let details = offer_details::Entity::find_by_id(id)
                .one(context.database)
                .await?;

            if details.is_none() || details.as_ref().and_then(|d| d.raw_data.as_ref()).is_none() {
                let offer_details = api_client_cloned.offer_details(&id).await?;
                if let Some(offer_details) = offer_details.body.response {
                    let active_model = converters::Database::convert_offer_details(&offer_details)?
                        .0
                        .into_active_model();
                    added_offers.push(active_model);
                }
            }
        }

        let mut events_to_dispatch = Vec::with_capacity(1 + added_offers.len().saturating_mul(2));

        events_to_dispatch.push(event::CreateEvent {
            event: Event::RefreshPoints { account_id },
            delay: Duration::from_secs(30),
        });

        for offer in &added_offers {
            let save_image_event = event::CreateEvent {
                event: Event::SaveImage {
                    basename: offer.image_base_name.as_ref().clone(),
                    force: *offer.migrated.as_ref(),
                },
                delay: Duration::ZERO,
            };

            let new_offer_event = event::CreateEvent {
                event: event::Event::NewOfferFound {
                    offer_proposition_id: *offer.proposition_id.as_ref(),
                },
                delay: Duration::from_secs(15),
            };

            events_to_dispatch.push(save_image_event);
            events_to_dispatch.push(new_offer_event);
        }

        context.set(RefreshContext { events_to_dispatch }).await?;

        let txn = context.database.begin().await?;

        offer_details::Entity::insert_many(added_offers)
            .on_conflict(
                OnConflict::column(offer_details::Column::PropositionId)
                    .update_columns(vec![
                        offer_details::Column::RawData,
                        offer_details::Column::ImageBaseName,
                    ])
                    .to_owned(),
            )
            .on_empty_do_nothing()
            .exec(&txn)
            .await?;

        let models = offer_list
            .offers
            .iter()
            .flat_map(|o| converters::Database::convert_offer(o, account_id))
            .map(|d| d.0.into_active_model())
            .collect::<Vec<_>>();

        offers::Entity::delete_many()
            .filter(offers::Column::AccountId.eq(account_id))
            .exec(&txn)
            .await?;

        let offer_history_models = models
            .iter()
            .cloned()
            .map(|m| -> Result<Database<offer_history::Model>, DbErr> {
                Ok(std::convert::Into::<Database<offer_history::Model>>::into(
                    Database::<offers::Model>(m.try_into_model()?),
                ))
            })
            .flat_map(|m| m.map(|r| r.0.into_active_model()))
            .collect::<Vec<_>>();

        offer_history::Entity::insert_many(offer_history_models)
            .on_empty_do_nothing()
            .exec(&txn)
            .await?;

        offers::Entity::insert_many(models)
            .on_empty_do_nothing()
            .exec(&txn)
            .await?;

        let now = chrono::offset::Utc::now().naive_utc();
        accounts::Entity::update(accounts::ActiveModel {
            id: sea_orm::Unchanged(account_id),
            refresh_failure_count: sea_orm::Set(0),
            offers_refreshed_at: sea_orm::Set(now),
            ..Default::default()
        })
        .exec(context.database)
        .await?;

        txn.commit().await?;

        // unlock account now if it was locked...
        let _ = account_lock::Entity::delete_by_id(account_id)
            .exec(context.database)
            .await;

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

        let http_client = get_http_client()?;
        let token =
            generate_internal_jwt(self.auth_secret.as_ref(), "Maccas Batch", "Maccas Event")?;

        self.create_bulk_events(&http_client, &token, refresh_context.events_to_dispatch)
            .await?;

        Ok(())
    }
}
