use aws_sdk_dynamodb::Client;
use config::Config;
use http::Method;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use maccas_core::cache;
use maccas_core::client;
use maccas_core::config::ApiConfig;
use maccas_core::utils;

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
    let client_map = client::get_client_map(&config, &client).await?;
    let params = request.path_parameters();
    let context = request.request_context();

    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => panic!(),
    };

    Ok(match resource_path {
        Some(s) => {
            let offer_map =
                cache::get_offers(&client, &config.cache_table_name, &client_map).await?;
            match s.as_str() {
                "/code/{dealId}" => {
                    let deal_id = params.first("dealId").expect("must have id");
                    let deal_id = &deal_id.to_owned();

                    match maccas_core::utils::get_by_order_id(&offer_map, deal_id, &client_map)
                        .await
                    {
                        Ok((api_client, _, _)) => {
                            let resp = api_client.offers_dealstack(None, None).await?;
                            serde_json::to_string(&resp).unwrap().into_response()
                        }

                        _ => Response::builder()
                            .status(400)
                            .body("".into())
                            .expect("failed to render response"),
                    }
                }

                "/deals/{dealId}" => {
                    let deal_id = params.first("dealId").expect("must have id");
                    let deal_id = &deal_id.to_owned();

                    match utils::get_by_order_id(&offer_map, deal_id, &client_map).await {
                        Ok((api_client, _, offer_proposition_id)) => match *request.method() {
                            Method::POST => {
                                let resp = api_client
                                    .add_offer_to_offers_dealstack(
                                        &offer_proposition_id,
                                        None,
                                        None,
                                    )
                                    .await?;

                                serde_json::to_string(&resp).unwrap().into_response()
                            }

                            Method::DELETE => {
                                let resp = api_client
                                    .remove_offer_from_offers_dealstack(
                                        deal_id.parse::<i64>().unwrap(),
                                        &offer_proposition_id,
                                        None,
                                        None,
                                    )
                                    .await?;

                                serde_json::to_string(&resp).unwrap().into_response()
                            }

                            _ => Response::builder()
                                .status(400)
                                .body("".into())
                                .expect("failed to render response"),
                        },

                        _ => Response::builder()
                            .status(400)
                            .body("".into())
                            .expect("failed to render response"),
                    }
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
