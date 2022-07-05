use crate::client;
use crate::constants::mc_donalds;
use crate::logging::log_deal_use;
use crate::routes::Context;
use crate::types;
use crate::types::api::{Error, OfferResponse};
use async_trait::async_trait;
use chrono::Duration;
use http::{Method, Response, StatusCode};
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct AddRemove;

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for AddRemove {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let path_params = request.path_parameters();

        let deal_id = path_params.first("dealId").expect("must have id");
        let deal_id = &deal_id.to_owned();

        if let Ok((account, offer)) = ctx.database.get_offer_by_id(deal_id).await {
            let query_params = request.query_string_parameters();
            let store = query_params.first("store");

            let http_client = client::get_http_client();
            let api_client = ctx
                .database
                .get_specific_client(
                    &http_client,
                    &ctx.config.client_id,
                    &ctx.config.client_secret,
                    &ctx.config.sensor_data,
                    &account,
                )
                .await?;

            let offer_id = offer.offer_id;
            let offer_proposition_id = offer.offer_proposition_id.to_string();
            let short_name = offer.short_name;

            Ok(match *request.method() {
                Method::POST => {
                    let resp = api_client
                        .add_to_offers_dealstack(
                            &offer_proposition_id,
                            mc_donalds::default::OFFSET,
                            store.unwrap_or(mc_donalds::default::STORE_ID),
                        )
                        .await?;

                    // this can cause the offer id to change.. for offers with id == 0
                    // we need to update the database to avoid inconsistency
                    if offer_id == 0 {
                        ctx.database.refresh_offer_cache_for(&account, &api_client).await?;
                    }

                    // lock the deal from appearing in GET /deals
                    ctx.database.lock_deal(deal_id, Duration::hours(12)).await?;

                    // if adding to the deal stack fails, we fail...
                    // we let the code above lock the deal though.
                    // likely case is someone redeeming a deal but also removing it..
                    // this lock will keep it removed and provide an error
                    // 409 Conflict means the offer already exists
                    // 404 when offer is already redeemed
                    if !resp.status.is_success() && resp.status.as_u16() != 409 {
                        return Ok(Response::builder().status(400).body(
                            serde_json::to_string(&types::api::Error {
                                message: "McDonald's API failed on deal stack operation.".to_string(),
                            })?
                            .into(),
                        )?);
                    }

                    // if we get 409 Conflict. offer already exists
                    let resp = if resp.status.as_u16() == 409 {
                        api_client
                            .get_offers_dealstack(
                                mc_donalds::default::OFFSET,
                                store.unwrap_or(mc_donalds::default::STORE_ID),
                            )
                            .await?
                    } else {
                        log_deal_use(
                            &http_client,
                            &request,
                            &short_name,
                            &deal_id,
                            &ctx.config.api_key,
                            &ctx.config.local_time_zone,
                        )
                        .await;
                        resp
                    };

                    let resp = OfferResponse::from(resp.body);
                    serde_json::to_value(&resp)?.into_response()
                }

                Method::DELETE => {
                    let resp = api_client
                        .remove_from_offers_dealstack(
                            &offer_id,
                            &offer_proposition_id,
                            mc_donalds::default::OFFSET,
                            store.unwrap_or(mc_donalds::default::STORE_ID),
                        )
                        .await?;

                    ctx.database.unlock_deal(deal_id).await?;
                    if resp.status.is_success() {
                        Response::builder().status(204).body(Body::Empty)?
                    } else {
                        Response::builder().status(400).body(
                            serde_json::to_string(&types::api::Error {
                                message: "McDonald's API failed on deal stack operation.".to_string(),
                            })?
                            .into(),
                        )?
                    }
                }

                _ => Response::builder().status(405).body(Body::Empty)?,
            })
        } else {
            let status_code = StatusCode::NOT_FOUND;
            Ok(Response::builder().status(status_code.as_u16()).body(
                serde_json::to_string(&Error {
                    message: status_code.canonical_reason().ok_or("no value")?.to_string(),
                })?
                .into(),
            )?)
        }
    }
}
