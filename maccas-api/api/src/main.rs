use aws_sdk_dynamodb::Client;
use chrono::Duration;
use core::cache;
use core::config;
use core::constants;
use core::lock;
use core::utils;
use http::Method;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use maccas_core::client;
use maccas_core::logging;

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

    Ok(match resource_path {
        Some(path) => {
            let path = path.as_str();

            let client = Client::new(&shared_config);
            let params = request.path_parameters();
            let query_params = request.query_string_parameters();

            let offer_map = cache::get_all_offers_as_map(&client, &config.cache_table_name).await?;
            let store = query_params.first("store");

            let deal_id = params.first("dealId").expect("must have id");
            let deal_id = &deal_id.to_owned();

            if let Ok((account_name, offer_proposition_id, offer_id)) =
                utils::get_by_order_id(&offer_map, deal_id).await
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
                            if offer_id == "0" {
                                cache::refresh_offer_cache_for(
                                    &client,
                                    &config.cache_table_name,
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
                                Duration::hours(6),
                            )
                            .await?;

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
                            let resp = api_client
                                .remove_offer_from_offers_dealstack(
                                    offer_id.parse::<i64>().unwrap(),
                                    &offer_proposition_id,
                                    None,
                                    store,
                                )
                                .await?;

                            lock::unlock_deal(&client, &config.offer_id_table_name, deal_id)
                                .await?;

                            serde_json::to_string(&resp).unwrap().into_response()
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
    })
}
