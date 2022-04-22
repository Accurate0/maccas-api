use aws_sdk_dynamodb::Client;
use config::Config;
use core::cache;
use core::client;
use core::config::ApiConfig;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
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
    let client_map = client::get_client_map(&config, &client).await?;
    let context = request.request_context();

    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => panic!(),
    };

    Ok(match resource_path {
        Some(s) => match s.as_str() {
            "/deals/refresh" => {
                cache::get_offers(&client, &config.cache_table_name, &client_map, true).await?;
                Response::builder()
                    .status(204)
                    .body("".into())
                    .expect("failed to render response")
            }

            "/deals" => {
                let offer_map =
                    cache::get_offers(&client, &config.cache_table_name, &client_map, false)
                        .await?;
                let mut offer_list = Vec::<Offer>::new();
                for (_, offers) in &offer_map {
                    offer_list.append(&mut offers.clone());
                }

                serde_json::to_string(&offer_list).unwrap().into_response()
            }

            _ => Response::builder()
                .status(400)
                .body("".into())
                .expect("failed to render response"),
        },
        None => Response::builder()
            .status(400)
            .body("".into())
            .expect("failed to render response"),
    })
}
