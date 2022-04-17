use aws_sdk_dynamodb::{model::AttributeValue, Client};
use chrono::{DateTime, FixedOffset, Utc};
use config::Config;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;
use std::collections::HashMap;
use std::time::SystemTime;

pub mod api;
pub mod api_types;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(run);
    lambda_runtime::run(func).await?;
    Ok(())
}

const VERSION: &str = "2";

async fn run(_: LambdaEvent<Value>) -> Result<(), Error> {
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

    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);

    let resp = client
        .get_item()
        .table_name(settings.get("tableName").unwrap().to_string())
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
                .table_name(settings.get("tableName").unwrap().to_string())
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

                    if diff.num_minutes() > 10 {
                        println!(">10 mins since last attempt.. refreshing..");
                        let mut new_access_token: String = String::from("");
                        let mut new_ref_token: String = String::from("");

                        let res = api_client.customer_login_refresh(refresh_token).await?;
                        if res.response.is_some() {
                            let unwrapped_res = res.response.unwrap();

                            new_access_token = unwrapped_res.access_token;
                            new_ref_token = unwrapped_res.refresh_token;
                        }

                        if res.status.code == 40000 {
                            api_client.security_auth_token().await?;
                            let res = api_client.customer_login().await?;

                            new_access_token = res.response.access_token;
                            new_ref_token = res.response.refresh_token;
                        }

                        client
                            .put_item()
                            .table_name(settings.get("tableName").unwrap().to_string())
                            .item("Version", AttributeValue::S(VERSION.to_owned()))
                            .item("access_token", AttributeValue::S(new_access_token))
                            .item("refresh_token", AttributeValue::S(new_ref_token))
                            .item("last_invocation", AttributeValue::S(now.to_rfc3339()))
                            .send()
                            .await?;
                    }
                }
                _ => panic!(),
            };
        }
    }

    let resp = api_client.get_offers(None).await?;
    println!("{:#?}", resp.status);

    Ok(())
}
