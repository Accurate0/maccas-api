use config::Config;
use http::Method;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use libmaccas::api::ApiClient;
use libmaccas::types::Offer;
use std::collections::HashMap;

pub mod client;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(run)).await?;
    Ok(())
}

#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
struct ApiConfigUsers {
    account_name: String,
    login_username: String,
    login_password: String,
}

#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
struct ApiConfig {
    client_id: String,
    client_secret: String,
    table_name: String,
    users: Vec<ApiConfigUsers>,
}

async fn get_by_order_id<'a>(
    deal_id: &String,
    client_map: &'a HashMap<String, ApiClient>,
) -> Result<(&'a ApiClient, String, String), Error> {
    let mut offer_map = HashMap::<&String, Vec<Offer>>::new();
    for (account_name, api_client) in client_map {
        let resp = api_client
            .get_offers(None)
            .await?
            .response
            .expect("to have response")
            .offers;

        offer_map.insert(account_name, resp);
    }

    let mut offer_account_name: Option<String> = None;
    let mut offer_proposition_id: Option<String> = None;
    for (account_name, offer_list) in offer_map {
        for offer in offer_list {
            if offer.offer_id.to_string() == *deal_id {
                offer_account_name = Some(account_name.to_string());
                offer_proposition_id = Some(offer.offer_proposition_id.to_string());
                break;
            }
        }
    }

    let offer_account_name = offer_account_name.ok_or("no account")?;
    let offer_proposition_id = offer_proposition_id.ok_or("no offer")?;
    let api_client = client_map.get(&offer_account_name).ok_or("no api client")?;

    Ok((api_client, offer_account_name, offer_proposition_id))
}

async fn run(request: Request) -> Result<impl IntoResponse, Error> {
    let config = Config::builder()
        .add_source(config::File::from_str(
            std::include_str!("config.yml"),
            config::FileFormat::Yaml,
        ))
        .build()
        .unwrap()
        .try_deserialize::<ApiConfig>()
        .expect("valid configuration present");

    let mut client_map = HashMap::<String, ApiClient>::new();
    for user in config.users {
        let api_client = client::get(
            &config.table_name,
            &user.account_name,
            &config.client_id,
            &config.client_secret,
            &user.login_username,
            &user.login_password,
        )
        .await?;

        client_map.insert(user.account_name, api_client);
    }

    let params = request.path_parameters();
    let context = request.request_context();

    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => panic!(),
    };

    Ok(match resource_path {
        Some(s) => match s.as_str() {
            "/deals" => {
                let mut offer_list = Vec::<Offer>::new();
                for (_account_name, api_client) in &client_map {
                    let mut resp = api_client
                        .get_offers(None)
                        .await?
                        .response
                        .expect("to have response")
                        .offers;

                    offer_list.append(&mut resp);
                    println!("{:#?}", resp);
                }

                serde_json::to_string(&offer_list).unwrap().into_response()
            }

            "/code/{dealId}" => {
                let deal_id = params.first("dealId").expect("must have id");
                let deal_id = &deal_id.to_owned();

                match get_by_order_id(deal_id, &client_map).await {
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

                match get_by_order_id(deal_id, &client_map).await {
                    Ok((api_client, _, offer_proposition_id)) => match *request.method() {
                        Method::POST => {
                            let resp = api_client
                                .add_offer_to_offers_dealstack(&offer_proposition_id, None, None)
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
        },
        None => Response::builder()
            .status(400)
            .body("".into())
            .expect("failed to render response"),
    })
}
