use crate::constants::{ACCESS_TOKEN, ACCOUNT_NAME, LAST_REFRESH, REFRESH_TOKEN};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, FixedOffset, Utc};
use lambda_http::Error;
use libmaccas::api;
use std::time::SystemTime;

pub async fn get(
    client: &aws_sdk_dynamodb::Client,
    table_name: &String,
    account_name: &String,
    client_id: &String,
    client_secret: &String,
    login_username: &String,
    login_password: &String,
) -> Result<api::ApiClient, Error> {
    let mut api_client = api::ApiClient::new(
        client_id.clone(),
        client_secret.clone(),
        login_username.clone(),
        login_password.clone(),
    );

    let resp = client
        .get_item()
        .table_name(table_name)
        .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
        .send()
        .await?;

    match resp.item {
        None => {
            println!("{}: nothing in db, requesting..", account_name);
            let _ = api_client.security_auth_token().await?;
            let response = api_client.customer_login().await?;

            let now = SystemTime::now();
            let now: DateTime<Utc> = now.into();
            let now = now.to_rfc3339();

            let resp = response.response;

            client
                .put_item()
                .table_name(table_name)
                .item(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
                .item(ACCESS_TOKEN, AttributeValue::S(resp.access_token))
                .item(REFRESH_TOKEN, AttributeValue::S(resp.refresh_token))
                .item(LAST_REFRESH, AttributeValue::S(now))
                .send()
                .await?;
        }
        Some(ref item) => {
            println!("{}: tokens in db, trying..", account_name);
            let refresh_token = match item[REFRESH_TOKEN].as_s() {
                Ok(s) => s,
                _ => panic!(),
            };

            match item[ACCESS_TOKEN].as_s() {
                Ok(s) => api_client.set_auth_token(s),
                _ => panic!(),
            };

            match item[LAST_REFRESH].as_s() {
                Ok(s) => {
                    let now = SystemTime::now();
                    let now: DateTime<Utc> = now.into();
                    let now: DateTime<FixedOffset> = DateTime::from(now);

                    let last_refresh = DateTime::parse_from_rfc3339(s).unwrap();

                    let diff = now - last_refresh;

                    if diff.num_minutes() > 9 {
                        println!(
                            "{}: >= 10 mins since last attempt.. refreshing..",
                            account_name
                        );
                        let mut new_access_token = String::from("");
                        let mut new_ref_token = String::from("");

                        let res = api_client.customer_login_refresh(refresh_token).await;
                        match res {
                            Ok(r) => {
                                if r.response.is_some() {
                                    let unwrapped_res = r.response.unwrap();
                                    println!("refresh success..");
                                    new_access_token = unwrapped_res.access_token;
                                    new_ref_token = unwrapped_res.refresh_token;
                                } else if r.status.code != 20000 {
                                    api_client.security_auth_token().await?;
                                    let res = api_client.customer_login().await?;
                                    println!("refresh failed, logged in again..");
                                    new_access_token = res.response.access_token;
                                    new_ref_token = res.response.refresh_token;
                                }

                                api_client.set_auth_token(&new_access_token);
                                client
                                    .put_item()
                                    .table_name(table_name)
                                    .item(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
                                    .item(ACCESS_TOKEN, AttributeValue::S(new_access_token))
                                    .item(REFRESH_TOKEN, AttributeValue::S(new_ref_token))
                                    .item(LAST_REFRESH, AttributeValue::S(now.to_rfc3339()))
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

    Ok(api_client)
}
