use chrono::Duration;
use core::cache;
use core::config;
use core::constants;
use core::lock;
use http::Method;
use itertools::Itertools;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use maccas_core::client;
use maccas_core::logging;
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use types::api::Offer;

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
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let context = request.request_context();
    let query_params = request.query_string_parameters();

    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => panic!(),
    };

    Ok(match resource_path {
        Some(s) => {
            let account_name_list: Vec<String> = config
                .users
                .iter()
                .map(|u| u.account_name.clone())
                .collect();

            match s.as_str() {
                "/locations" => {
                    let distance = query_params.first("distance");
                    let latitude = query_params.first("latitude");
                    let longitude = query_params.first("longitude");

                    if distance.is_some() && latitude.is_some() && longitude.is_some() {
                        let mut rng = StdRng::from_entropy();
                        let choice = account_name_list.choose(&mut rng).unwrap().to_string();
                        let user = config
                            .users
                            .iter()
                            .find(|u| u.account_name == choice)
                            .unwrap();

                        let http_client = client::get_http_client();
                        let api_client = core::client::get(
                            &http_client,
                            &dynamodb_client,
                            &choice,
                            &config,
                            &user.login_username,
                            &user.login_password,
                        )
                        .await?;
                        let resp = api_client
                            .restaurant_location(distance, latitude, longitude, None)
                            .await?;

                        serde_json::to_string(&resp).unwrap().into_response()
                    } else {
                        Response::builder().status(400).body("".into()).unwrap()
                    }
                }

                "/deals" => {
                    let locked_deals =
                        lock::get_all_locked_deals(&dynamodb_client, &config.offer_id_table_name)
                            .await?;

                    let offer_list =
                        cache::get_all_offers_as_vec(&dynamodb_client, &config.cache_table_name)
                            .await?;

                    // filter locked deals & extras
                    // 30762 is McCafé®, Buy 5 Get 1 Free, valid till end of year...
                    let offer_list: Vec<Offer> = offer_list
                        .into_iter()
                        .filter(|offer| {
                            !locked_deals.contains(&offer.deal_uuid.to_string())
                                && offer.offer_proposition_id != 30762
                        })
                        .collect();

                    let mut count_map = HashMap::<i64, u32>::new();
                    for offer in &offer_list {
                        match count_map.get(&offer.offer_proposition_id) {
                            Some(count) => {
                                let count = count + 1;
                                count_map.insert(offer.offer_proposition_id.clone(), count)
                            }
                            None => count_map.insert(offer.offer_proposition_id.clone(), 1),
                        };
                    }

                    let offer_list: Vec<Offer> = offer_list
                        .into_iter()
                        .unique_by(|offer| offer.offer_proposition_id)
                        .map(|mut offer| {
                            offer.count = *count_map.get(&offer.offer_proposition_id).unwrap();
                            offer
                        })
                        .collect();

                    serde_json::to_string(&offer_list).unwrap().into_response()
                }

                "/deals/lock/{dealId}" => {
                    let params = request.path_parameters();

                    let deal_id = params.first("dealId").expect("must have id");
                    let deal_id = &deal_id.to_owned();

                    match *request.method() {
                        Method::POST => {
                            let duration =
                                query_params.first("duration").expect("must have duration");
                            lock::lock_deal(
                                &dynamodb_client,
                                &config.offer_id_table_name,
                                deal_id,
                                Duration::seconds(duration.parse::<i64>().unwrap()),
                            )
                            .await?;
                            Response::builder().status(204).body("".into()).unwrap()
                        }
                        Method::DELETE => {
                            lock::unlock_deal(
                                &dynamodb_client,
                                &config.offer_id_table_name,
                                deal_id,
                            )
                            .await?;
                            Response::builder().status(204).body("".into()).unwrap()
                        }
                        _ => Response::builder().status(400).body("".into()).unwrap(),
                    }
                }
                _ => Response::builder().status(400).body("".into()).unwrap(),
            }
        }
        None => Response::builder().status(400).body("".into()).unwrap(),
    })
}
