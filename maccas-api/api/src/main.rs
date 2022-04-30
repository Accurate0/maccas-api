use aws_sdk_dynamodb::Client;
use core::cache;
use core::client;
use core::utils;
use http::Method;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};

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

    let context = request.request_context();
    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => panic!(),
    };

    Ok(match resource_path {
        Some(path) => {
            let path = path.as_str();

            let shared_config = aws_config::load_from_env().await;
            let client = Client::new(&shared_config);
            let params = request.path_parameters();
            let query_params = request.query_string_parameters();
            let account_name_list = config
                .users
                .iter()
                .map(|u| u.account_name.clone())
                .collect();

            let offer_map =
                cache::get_offers(&client, &config.cache_table_name, &account_name_list).await?;
            let store = query_params.first("store");

            let deal_id = params.first("dealId").expect("must have id");
            let deal_id = &deal_id.to_owned();

            let account_name_and_offer_id = utils::get_by_order_id(&offer_map, deal_id).await;

            match account_name_and_offer_id {
                Ok((account_name, offer_proposition_id)) => {
                    let user = config
                        .users
                        .iter()
                        .find(|u| u.account_name == account_name)
                        .unwrap();

                    let api_client = client::get(
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
                                    .add_offer_to_offers_dealstack(
                                        &offer_proposition_id,
                                        None,
                                        store,
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
                                        store,
                                    )
                                    .await?;

                                serde_json::to_string(&resp).unwrap().into_response()
                            }

                            _ => Response::builder().status(400).body("".into()).unwrap(),
                        },

                        // this isn't something that will happen
                        _ => Response::builder().status(401).body("".into()).unwrap(),
                    }
                }
                _ => Response::builder().status(400).body("".into()).unwrap(),
            }
        }
        _ => Response::builder().status(400).body("".into()).unwrap(),
    })
}
