use super::{Job, JobContext};
use crate::settings::McDonalds;
use base::constants::mc_donalds;
use entity::{accounts, offer_details, offers, points};
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{
    sea_query::OnConflict, ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct RefreshJob {
    pub http_client: ClientWithMiddleware,
    pub mcdonalds_config: McDonalds,
}

#[async_trait::async_trait]
impl Job for RefreshJob {
    fn name(&self) -> String {
        "refresh".to_owned()
    }

    // TODO: needs refreshed at datetime as well, since updated at is updated by updating tokens alone
    // that can happen at any point really
    async fn execute(&self, context: &JobContext, _cancellation_token: CancellationToken) {
        let func = async {
            let account_to_refresh = accounts::Entity::find()
                .order_by_asc(accounts::Column::UpdatedAt)
                .one(&context.database)
                .await?
                .ok_or_else(|| anyhow::Error::msg("no account found"))?;

            let account_id = account_to_refresh.id.to_owned();
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

            update_model.update(&context.database).await?;

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

                        if details.is_none() {
                            let offer_details = api_client_cloned.offer_details(&id).await?;
                            if let Some(offer_details) = offer_details.body.response {
                                return Ok(Some(
                                    converters::Database::convert_offer_details(&offer_details)?
                                        .0
                                        .into_active_model(),
                                ));
                            }
                        }

                        // TODO: get image for each offer details
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

            let txn = context.database.begin().await?;

            offer_details::Entity::insert_many(active_models)
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

            offers::Entity::insert_many(models)
                .on_empty_do_nothing()
                .exec(&txn)
                .await?;

            txn.commit().await?;

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

            Ok::<(), anyhow::Error>(())
        };

        match func.await {
            Ok(_) => {}
            Err(e) => {
                tracing::error!("error while executing iteration: {}", e)
            }
        }
    }

    async fn cleanup(&self, _context: &JobContext) {}
}
