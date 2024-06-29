use super::{error::JobError, Job, JobContext, JobType};
use crate::settings::McDonalds;
use base::{constants::mc_donalds, http::get_http_client, jwt::generate_internal_jwt};
use converters::Database;
use entity::{account_lock, accounts, offer_details, offer_history, offers, points};
use event::{CreateEventResponse, Event};
use libmaccas::ApiClient;
use reqwest::StatusCode;
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, QuerySelect, Set, TransactionTrait, TryIntoModel,
};
use serde::Serialize;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct RefreshJob {
    pub event_api_base: String,
    pub http_client: ClientWithMiddleware,
    pub auth_secret: String,
    pub mcdonalds_config: McDonalds,
}

#[derive(Serialize)]
struct RefreshContext {
    account_id: String,
}

#[async_trait::async_trait]
impl Job for RefreshJob {
    fn name(&self) -> String {
        "refresh".to_owned()
    }

    fn job_type(&self) -> JobType {
        JobType::Schedule("0 */5 * * * *".parse().unwrap())
    }

    // TODO: needs refreshed at datetime as well, since updated at is updated by updating tokens alone
    // that can happen at any point really
    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let txn = context.database.begin().await?;

        let account_to_refresh = accounts::Entity::find()
            .lock_exclusive()
            .filter(accounts::Column::Active.eq(true))
            .filter(accounts::Column::RefreshFailureCount.lte(3))
            .order_by_asc(accounts::Column::UpdatedAt)
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow::Error::msg("no account found"))?;

        context
            .set(RefreshContext {
                account_id: account_to_refresh.id.to_string(),
            })
            .await?;

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

            update_model.update(&txn).await?;

            Ok::<ApiClient, anyhow::Error>(api_client)
        }
        .await;

        if let Err(e) = api_client {
            tracing::warn!("increasing error count: {}", current_failure_count + 1);
            accounts::Entity::update(accounts::ActiveModel {
                id: sea_orm::Unchanged(account_id),
                refresh_failure_count: sea_orm::Set(current_failure_count + 1),
                ..Default::default()
            })
            .exec(&txn)
            .await?;

            txn.commit().await?;

            return Err(e.into());
        }

        txn.commit().await?;
        let api_client = api_client.unwrap();

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

        let proposition_id_futures = offer_list
            .offers
            .iter()
            .map(|o| o.offer_proposition_id)
            .map(|id| {
                let api_client_cloned = api_client.clone();

                async move {
                    let details = offer_details::Entity::find_by_id(id)
                        .one(&context.database)
                        .await?;

                    if details.is_none()
                        || details.as_ref().and_then(|d| d.raw_data.as_ref()).is_none()
                    {
                        let offer_details = api_client_cloned.offer_details(&id).await?;
                        if let Some(offer_details) = offer_details.body.response {
                            return Ok(Some(
                                converters::Database::convert_offer_details(&offer_details)?
                                    .0
                                    .into_active_model(),
                            ));
                        }
                    }

                    Ok::<Option<offer_details::ActiveModel>, anyhow::Error>(None)
                }
            })
            .collect::<Vec<_>>();

        let completed_futures = futures::future::join_all(proposition_id_futures).await;
        let mut active_models = Vec::new();
        for active_model in completed_futures {
            match active_model {
                Ok(m) => {
                    if let Some(model) = m {
                        active_models.push(model);
                    }
                }
                Err(e) => tracing::error!("error fetching offer details: {}", e),
            }
        }

        let http_client = get_http_client()?;
        let token =
            generate_internal_jwt(self.auth_secret.as_ref(), "Maccas Batch", "Maccas Event")?;
        let request_url = format!("{}/{}", self.event_api_base, event::CreateEvent::path());

        for offer in &active_models {
            let save_image_event = event::CreateEvent {
                event: Event::SaveImage {
                    basename: offer.image_base_name.as_ref().clone(),
                },
                delay: Duration::from_secs(0),
            };

            let request = http_client
                .post(&request_url)
                .json(&save_image_event)
                .bearer_auth(&token);

            let response = request.send().await;

            match response {
                Ok(response) => match response.status() {
                    StatusCode::CREATED => {
                        let id = response.json::<CreateEventResponse>().await?.id;
                        tracing::info!("created image event with id {}", id);
                    }
                    status => {
                        tracing::warn!("event failed with {} - {}", status, response.text().await?);
                    }
                },
                Err(e) => tracing::warn!("event request failed with {}", e),
            }
        }

        let txn = context.database.begin().await?;

        offer_details::Entity::insert_many(active_models)
            .on_conflict(
                OnConflict::column(offer_details::Column::PropositionId)
                    .update_column(offer_details::Column::RawData)
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

        accounts::Entity::update(accounts::ActiveModel {
            id: sea_orm::Unchanged(account_id),
            refresh_failure_count: sea_orm::Set(0),
            ..Default::default()
        })
        .exec(&txn)
        .await?;

        txn.commit().await?;

        // unlock account now if it was locked...
        let _ = account_lock::Entity::delete_by_id(account_id)
            .exec(&context.database)
            .await;

        let txn = context.database.begin().await?;

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
}
