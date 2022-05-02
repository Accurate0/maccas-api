use aws_sdk_dynamodb::Client;
use chrono::Duration;
use config::Config;
use core::cache;
use core::client;
use core::config::ApiConfig;
use core::lock;
use http::Method;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use rand::prelude::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;
use types::maccas::Offer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(run)).await?;
    Ok(())
}

async fn run(request: Request) -> Result<impl IntoResponse, Error> {
    let config = Config::builder()
        .add_source(config::File::from_str(
            std::include_str!("../../base-config.yml"),
            config::FileFormat::Yaml,
        ))
        .add_source(config::File::from_str(
            std::include_str!("../../accounts-all.yml"),
            config::FileFormat::Yaml,
        ))
        .build()
        .unwrap()
        .try_deserialize::<ApiConfig>()
        .expect("valid configuration present");

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
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

                        let api_client = client::get(
                            &client,
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
                        lock::get_all_locked_deals(&client, &config.offer_id_table_name).await?;

                    let offer_map =
                        cache::get_offers(&client, &config.cache_table_name, &account_name_list)
                            .await?;
                    let mut offer_list = Vec::<Offer>::new();
                    for (_, offers) in &offer_map {
                        match offers {
                            Some(offers) => {
                                offer_list.append(&mut offers.clone());
                            }
                            None => {}
                        }
                    }

                    // filter locked deals & extras
                    // 30762 is McCafé®, Buy 5 Get 1 Free, valid till end of year...
                    let offer_list: Vec<&Offer> = offer_list
                        .iter()
                        .filter(|offer| {
                            !locked_deals.contains(&offer.offer_id.to_string())
                                && offer.offer_proposition_id != 30762
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
                                &client,
                                &config.offer_id_table_name,
                                deal_id,
                                Duration::seconds(duration.parse::<i64>().unwrap()),
                            )
                            .await?;
                            Response::builder().status(204).body("".into()).unwrap()
                        }
                        Method::DELETE => {
                            lock::unlock_deal(&client, &config.offer_id_table_name, deal_id)
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
