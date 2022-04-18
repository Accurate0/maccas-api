use aws_sdk_dynamodb::{model::AttributeValue, Client};
use chrono::{DateTime, FixedOffset, Utc};
use config::Config;
use http::Method;
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;

pub mod api;
pub mod api_types;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(run)).await?;
    Ok(())
}

const VERSION: &str = "3";

async fn run(request: Request) -> Result<impl IntoResponse, Error> {
    println!("{:#?}", request);
    let settings = Config::builder()
        .add_source(config::File::from_str(
            std::include_str!("config.yml"),
            config::FileFormat::Yaml,
        ))
        .add_source(config::Environment::with_prefix("MCD_API"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let mut api_client = api::ApiClient::new(
        settings.get("clientId").unwrap().to_string(),
        settings.get("clientSecret").unwrap().to_string(),
        settings.get("loginUsername").unwrap().to_string(),
        settings.get("loginPassword").unwrap().to_string(),
    );

    let table_name = settings.get("tableName").unwrap().to_string();
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    let resp = client
        .get_item()
        .table_name(&table_name)
        .key("Version", AttributeValue::S(VERSION.to_string()))
        .send()
        .await?;

    match resp.item {
        None => {
            println!("nothing in db, requesting..");
            let _ = api_client.security_auth_token().await?;
            let response = api_client.customer_login().await?;

            let now = SystemTime::now();
            let now: DateTime<Utc> = now.into();
            let now = now.to_rfc3339();

            client
                .put_item()
                .table_name(&table_name)
                .item("Version", AttributeValue::S(VERSION.to_owned()))
                .item(
                    "access_token",
                    AttributeValue::S(response.response.access_token),
                )
                .item(
                    "refresh_token",
                    AttributeValue::S(response.response.refresh_token),
                )
                .item("last_invocation", AttributeValue::S(now))
                .send()
                .await?;
        }
        Some(ref item) => {
            println!("tokens in db, trying..");
            let refresh_token = match item["refresh_token"].as_s() {
                Ok(s) => s,
                _ => panic!(),
            };

            match item["access_token"].as_s() {
                Ok(s) => api_client.set_auth_token(s),
                _ => panic!(),
            };

            match item["last_invocation"].as_s() {
                Ok(s) => {
                    let now = SystemTime::now();
                    let now: DateTime<Utc> = now.into();
                    let now: DateTime<FixedOffset> = DateTime::from(now);

                    let last_invocation = DateTime::parse_from_rfc3339(s).unwrap();

                    let diff = now - last_invocation;

                    if diff.num_minutes() > 9 {
                        println!(">= 10 mins since last attempt.. refreshing..");
                        let mut new_access_token = String::from("");
                        let mut new_ref_token = String::from("");

                        let res = api_client.customer_login_refresh(refresh_token).await;
                        match res {
                            Ok(r) => {
                                if r.response.is_some() {
                                    let unwrapped_res = r.response.unwrap();

                                    new_access_token = unwrapped_res.access_token;
                                    new_ref_token = unwrapped_res.refresh_token;
                                } else if r.status.code != 20000 {
                                    api_client.security_auth_token().await?;
                                    let res = api_client.customer_login().await?;

                                    new_access_token = res.response.access_token;
                                    new_ref_token = res.response.refresh_token;
                                }

                                api_client.set_auth_token(&new_access_token);
                                client
                                    .put_item()
                                    .table_name(&table_name)
                                    .item("Version", AttributeValue::S(VERSION.to_owned()))
                                    .item("access_token", AttributeValue::S(new_access_token))
                                    .item("refresh_token", AttributeValue::S(new_ref_token))
                                    .item("last_invocation", AttributeValue::S(now.to_rfc3339()))
                                    .send()
                                    .await?;
                            }

                            Err(_) => panic!(),
                        }
                    }
                }
                _ => panic!(),
            };
        }
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
                let resp = api_client
                    .get_offers(None)
                    .await?
                    .response
                    .expect("to have response")
                    .offers;

                serde_json::to_string(&resp).unwrap().into_response()
            }

            "/deals/{dealId}" => {
                let deal_id = params.first("dealId").expect("must have id");
                let deal_id = &deal_id.to_owned();

                match *request.method() {
                    Method::POST => {
                        let resp = api_client
                            .add_offer_to_offers_dealstack(deal_id, None, None)
                            .await?;

                        serde_json::to_string(&resp).unwrap().into_response()
                    }

                    Method::DELETE => {
                        let resp = api_client
                            .get_offers(None)
                            .await?
                            .response
                            .expect("to have response")
                            .offers;

                        let offer_id_vec: Vec<i64> = resp
                            .iter()
                            .filter(|d| d.offer_proposition_id.to_string() == *deal_id)
                            .map(|d| d.offer_id)
                            .collect();

                        let offer_id = offer_id_vec.first().unwrap();

                        let resp = api_client
                            .remove_offer_from_offers_dealstack(*offer_id, deal_id, None, None)
                            .await?;

                        serde_json::to_string(&resp).unwrap().into_response()
                    }

                    _ => panic!(),
                }
            }

            _ => Response::builder()
                .status(400)
                .body("Bad Request".into())
                .expect("failed to render response"),
        },
        None => Response::builder()
            .status(400)
            .body("Bad Request".into())
            .expect("failed to render response"),
    })
}
