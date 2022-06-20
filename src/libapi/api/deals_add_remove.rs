use super::Context;
use crate::cache;
use crate::client::{self};
use crate::constants::{api_base, mc_donalds};
use crate::extensions::RequestExtensions;
use crate::types::api::{Error, OfferResponse};
use crate::types::jwt::JwtClaim;
use crate::types::log::UsageLog;
use crate::{constants, lock};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use http::{Method, Response, StatusCode};
use jwt::{Header, Token};
use lambda_http::{Body, IntoResponse, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct DealsAddRemove;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for DealsAddRemove {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let path_params = request.path_parameters();

        let deal_id = path_params.first("dealId").expect("must have id");
        let deal_id = &deal_id.to_owned();

        if let Ok((account_name, offer)) =
            cache::get_offer_by_id(deal_id, &ctx.dynamodb_client, &ctx.config.cache_table_name_v2).await
        {
            let query_params = request.query_string_parameters();
            let store = query_params.first("store");

            let user = ctx
                .config
                .users
                .iter()
                .find(|u| u.account_name == account_name)
                .ok_or("no account found")?;

            let http_client = client::get_http_client();
            let api_client = client::get(
                &http_client,
                &ctx.dynamodb_client,
                &account_name,
                &ctx.config,
                &user.login_username,
                &user.login_password,
            )
            .await?;

            let offer_id = offer.offer_id;
            let offer_proposition_id = offer.offer_proposition_id.to_string();
            let offer_name = offer.name;

            Ok(match *request.method() {
                Method::POST => {
                    let resp = api_client
                        // let store_id = store_id.unwrap_or("951488");
                        // let offset = offset.unwrap_or("480").to_string();
                        .add_to_offers_dealstack(
                            &offer_proposition_id,
                            mc_donalds::default::OFFSET,
                            store.unwrap_or(mc_donalds::default::STORE_ID),
                        )
                        .await?;
                    // this can cause the offer id to change.. for offers with id == 0
                    // we need to update the database to avoid inconsistency
                    if offer_id == 0 {
                        cache::refresh_offer_cache_for(
                            &ctx.dynamodb_client,
                            &ctx.config.cache_table_name,
                            &ctx.config.cache_table_name_v2,
                            &account_name,
                            &api_client,
                        )
                        .await?;
                    }

                    // lock the deal from appearing in GET /deals
                    lock::lock_deal(
                        &ctx.dynamodb_client,
                        &ctx.config.offer_id_table_name,
                        deal_id,
                        Duration::hours(3),
                    )
                    .await?;

                    // log usage
                    let auth_header = request.headers().get(http::header::AUTHORIZATION);
                    if let Some(auth_header) = auth_header {
                        let value = auth_header.to_str()?.replace("Bearer ", "");
                        let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value)?;
                        let correlation_id = request.get_correlation_id();
                        let dt: DateTime<Local> = Local::now();

                        let usage_log = UsageLog {
                            user_id: jwt.claims().oid.to_string(),
                            deal_readable: offer_name.split("\n").collect::<Vec<&str>>()[0].to_string(),
                            deal_uuid: deal_id.to_string(),
                            user_readable: jwt.claims().name.to_string(),
                            message: "Deal Used",
                            local_time: dt.format("%a %b %e %T %Y").to_string(),
                        };

                        let response = http_client
                            .request(Method::POST, format!("{}/log", api_base::LOG).as_str())
                            .header(constants::LOG_SOURCE_HEADER, constants::SOURCE_NAME)
                            .header(constants::CORRELATION_ID_HEADER, correlation_id)
                            .header(constants::X_API_KEY_HEADER, &ctx.config.api_key)
                            .body(serde_json::to_string(&usage_log)?)
                            .send()
                            .await;

                        log::info!("logging response: {:#?}", response);
                    }

                    // if its none, this offer already exists, but we should provide the deal stack information
                    // idempotent
                    let resp = if resp.response.is_none() {
                        api_client
                            .get_offers_dealstack(
                                mc_donalds::default::OFFSET,
                                store.unwrap_or(mc_donalds::default::STORE_ID),
                            )
                            .await?
                    } else {
                        resp
                    };

                    let resp = OfferResponse::from(resp);
                    serde_json::to_value(&resp)?.into_response()
                }

                Method::DELETE => {
                    // let store_id = store_id.unwrap_or("951488");
                    // let offset = offset.unwrap_or(480).to_string();
                    api_client
                        .remove_from_offers_dealstack(
                            &offer_id,
                            &offer_proposition_id,
                            mc_donalds::default::OFFSET,
                            store.unwrap_or(mc_donalds::default::STORE_ID),
                        )
                        .await?;

                    lock::unlock_deal(&ctx.dynamodb_client, &ctx.config.offer_id_table_name, deal_id).await?;

                    Response::builder().status(204).body(Body::Empty)?
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
