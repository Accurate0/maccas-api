use aws_sdk_dynamodb::Client;
use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use core::cache;
use core::config;
use core::constants;
use core::lock;
use http::HeaderValue;
use http::Method;
use jwt::Header;
use jwt::Token;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use libmaccas::util;
use maccas_core::client;
use maccas_core::logging;
use types::bot::UsageLog;

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::setup_logging();
    lambda_http::run(service_fn(run)).await?;
    Ok(())
}

async fn run(request: Request) -> Result<impl IntoResponse, Error> {
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;
    let config = config::load_from_s3(&shared_config).await;
    let context = request.request_context();
    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => panic!(),
    };

    log::info!("request: {:#?}", request);

    let response = match resource_path {
        Some(path) => {
            let path = path.as_str();

            let client = Client::new(&shared_config);
            let params = request.path_parameters();
            let query_params = request.query_string_parameters();

            let store = query_params.first("store");
            let deal_id = params.first("dealId").expect("must have id");
            let deal_id = &deal_id.to_owned();

            if let Some((account_name, offer)) =
                cache::get_offer_by_id(deal_id, &client, &config.cache_table_name_v2).await
            {
                let user = config
                    .users
                    .iter()
                    .find(|u| u.account_name == account_name)
                    .unwrap();

                let http_client = client::get_http_client();
                let api_client = core::client::get(
                    &http_client,
                    &client,
                    &account_name,
                    &config,
                    &user.login_username,
                    &user.login_password,
                )
                .await?;

                let offer_id = offer.offer_id;
                let offer_proposition_id = offer.offer_proposition_id.to_string();
                let offer_name = offer.name;

                match path {
                    "/code/{dealId}" => {
                        let resp = api_client.offers_dealstack(None, store).await?;
                        serde_json::to_string(&resp).unwrap().into_response()
                    }

                    "/deals/{dealId}" => match *request.method() {
                        Method::POST => {
                            let resp = api_client
                                .add_offer_to_offers_dealstack(&offer_proposition_id, None, store)
                                .await?;
                            // this can cause the offer id to change.. for offers with id == 0
                            // we need to update the database to avoid inconsistency
                            if offer_id == 0 {
                                cache::refresh_offer_cache_for(
                                    &client,
                                    &config.cache_table_name,
                                    &config.cache_table_name_v2,
                                    &account_name,
                                    &api_client,
                                )
                                .await?;
                            }

                            // lock the deal from appearing in GET /deals
                            lock::lock_deal(
                                &client,
                                &config.offer_id_table_name,
                                deal_id,
                                Duration::hours(3),
                            )
                            .await?;

                            // log usage
                            let auth_header = request.headers().get(http::header::AUTHORIZATION);
                            if let Some(auth_header) = auth_header {
                                let value = auth_header.to_str().unwrap().replace("Bearer ", "");
                                let jwt: Token<Header, types::jwt::JwtClaim, _> =
                                    jwt::Token::parse_unverified(&value).unwrap();
                                let potential_header =
                                    HeaderValue::from_str(util::get_uuid().as_str()).unwrap();
                                let correlation_id = request
                                    .headers()
                                    .get(constants::CORRELATION_ID_HEADER)
                                    .unwrap_or_else(|| &potential_header);
                                let dt: DateTime<Local> = Local::now();

                                let usage_log = UsageLog {
                                    user_id: jwt.claims().oid.to_string(),
                                    deal_readable: offer_name.split("\n").collect::<Vec<&str>>()[0]
                                        .to_string(),
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

                            lock::unlock_deal(&client, &config.offer_id_table_name, deal_id)
                                .await?;

                            Response::builder().status(204).body("".into()).unwrap()
                        }

                        _ => Response::builder().status(405).body("".into()).unwrap(),
                    },

                    // this isn't something that will happen
                    _ => Response::builder().status(400).body("".into()).unwrap(),
                }
            } else {
                Response::builder().status(400).body("".into()).unwrap()
            }
        }
        _ => Response::builder().status(400).body("".into()).unwrap(),
    };

    log::info!("response: {:#?}", response);
    Ok(response)
}
