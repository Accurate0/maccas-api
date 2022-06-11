use crate::client::{self};
use crate::extensions::RequestExtensions;
use crate::routes::Route;
use crate::types::jwt::JwtClaim;
use crate::types::log::UsageLog;
use crate::{cache, config::ApiConfig};
use crate::{constants, lock};
use async_trait::async_trait;
use chrono::{DateTime, Duration, Local};
use http::{Method, Response};
use jwt::{Header, Token};
use lambda_http::{Body, Error, IntoResponse, Request, RequestExt};

pub struct DealsAddRemove;

#[async_trait]
impl Route for DealsAddRemove {
    async fn execute(
        request: &Request,
        dynamodb_client: &aws_sdk_dynamodb::Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
        let path_params = request.path_parameters();
        let query_params = request.query_string_parameters();

        let store = query_params.first("store");
        let deal_id = path_params.first("dealId").expect("must have id");
        let deal_id = &deal_id.to_owned();

        let (account_name, offer) =
            cache::get_offer_by_id(deal_id, &dynamodb_client, &config.cache_table_name_v2).await?;
        let user = config
            .users
            .iter()
            .find(|u| u.account_name == account_name)
            .unwrap();

        let http_client = client::get_http_client();
        let api_client = client::get(
            &http_client,
            &dynamodb_client,
            &account_name,
            &config,
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
                    .add_offer_to_offers_dealstack(&offer_proposition_id, None, store)
                    .await?;
                // this can cause the offer id to change.. for offers with id == 0
                // we need to update the database to avoid inconsistency
                if offer_id == 0 {
                    cache::refresh_offer_cache_for(
                        &dynamodb_client,
                        &config.cache_table_name,
                        &config.cache_table_name_v2,
                        &account_name,
                        &api_client,
                    )
                    .await?;
                }

                // lock the deal from appearing in GET /deals
                lock::lock_deal(
                    &dynamodb_client,
                    &config.offer_id_table_name,
                    deal_id,
                    Duration::hours(3),
                )
                .await?;

                // log usage
                let auth_header = request.headers().get(http::header::AUTHORIZATION);
                if let Some(auth_header) = auth_header {
                    let value = auth_header.to_str().unwrap().replace("Bearer ", "");
                    let jwt: Token<Header, JwtClaim, _> =
                        jwt::Token::parse_unverified(&value).unwrap();
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
                        .request(
                            Method::POST,
                            format!("{}/log", constants::LOG_API_BASE).as_str(),
                        )
                        .header(constants::LOG_SOURCE_HEADER, constants::SOURCE_NAME)
                        .header(constants::CORRELATION_ID_HEADER, correlation_id)
                        .header(constants::X_API_KEY_HEADER, &config.api_key)
                        .body(serde_json::to_string(&usage_log).unwrap())
                        .send()
                        .await;

                    log::info!("logging response: {:#?}", response);
                }

                // if its none, this offer already exists, but we should provide the deal stack information
                // idempotent
                if resp.response.is_none() {
                    let resp = api_client.offers_dealstack(None, store).await?;
                    serde_json::to_string(&resp).unwrap().into_response()
                } else {
                    serde_json::to_string(&resp).unwrap().into_response()
                }
            }

            Method::DELETE => {
                api_client
                    .remove_offer_from_offers_dealstack(
                        offer_id,
                        &offer_proposition_id,
                        None,
                        store,
                    )
                    .await?;

                lock::unlock_deal(&dynamodb_client, &config.offer_id_table_name, deal_id).await?;

                Response::builder().status(204).body("".into()).unwrap()
            }

            _ => Response::builder().status(405).body("".into()).unwrap(),
        })
    }
}
