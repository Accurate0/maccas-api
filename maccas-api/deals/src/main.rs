use aws_sdk_dynamodb::Client;
use config::Config;
use core::cache;
use core::client;
use core::config::ApiConfig;
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
            std::include_str!("../../config.yml"),
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
                        Response::builder()
                            .status(400)
                            .body("".into())
                            .expect("failed to render response")
                    }
                }

                "/deals/refresh" => {
                    let client_map = client::get_client_map(&config, &client).await?;
                    cache::refresh_offer_cache(
                        &client,
                        &config.cache_table_name,
                        &client_map,
                        true,
                    )
                    .await?;
                    Response::builder()
                        .status(204)
                        .body("".into())
                        .expect("failed to render response")
                }

                "/deals" => {
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

                    serde_json::to_string(&offer_list).unwrap().into_response()
                }

                _ => Response::builder()
                    .status(400)
                    .body("".into())
                    .expect("failed to render response"),
            }
        }
        None => Response::builder()
            .status(400)
            .body("".into())
            .expect("failed to render response"),
    })
}
