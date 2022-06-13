use crate::client;
use crate::config::ApiConfig;
use crate::constants::{ACCESS_TOKEN, ACCOUNT_NAME, LAST_REFRESH, MCDONALDS_API_BASE_URL, REFRESH_TOKEN};
use crate::middleware;
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, FixedOffset, Utc};
use lambda_http::Error;
use libmaccas::ApiClient;
use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;

pub fn get_http_client() -> reqwest_middleware::ClientWithMiddleware {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    middleware::get_middleware_http_client(client)
}

pub fn get_http_client_with_headers(headers: http::HeaderMap) -> reqwest_middleware::ClientWithMiddleware {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .build()
        .unwrap();
    middleware::get_middleware_http_client(client)
}

pub async fn get_client_map<'a>(
    http_client: &'a reqwest_middleware::ClientWithMiddleware,
    config: &'a ApiConfig,
    client: &'a aws_sdk_dynamodb::Client,
) -> Result<HashMap<String, ApiClient<'a>>, Error> {
    let mut client_map = HashMap::<String, ApiClient<'_>>::new();
    for user in &config.users {
        let api_client = client::get(
            http_client,
            &client,
            &user.account_name,
            &config,
            &user.login_username,
            &user.login_password,
        )
        .await?;

        client_map.insert(user.account_name.clone(), api_client);
    }

    Ok(client_map)
}

pub async fn get<'a>(
    http_client: &'a reqwest_middleware::ClientWithMiddleware,
    client: &'a aws_sdk_dynamodb::Client,
    account_name: &'a String,
    config: &'a ApiConfig,
    login_username: &'a String,
    login_password: &'a String,
) -> Result<ApiClient<'a>, Error> {
    let mut api_client = ApiClient::new(
        MCDONALDS_API_BASE_URL.to_string(),
        http_client,
        config.client_id.clone(),
    );

    let resp = client
        .get_item()
        .table_name(&config.table_name)
        .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
        .send()
        .await?;

    match resp.item {
        None => {
            log::info!("{}: nothing in db, requesting..", account_name);
            let response = api_client.security_auth_token(&config.client_secret).await?;
            api_client.set_login_token(&response.response.token);

            let response = api_client.customer_login(login_username, login_password).await?;
            api_client.set_auth_token(&response.response.access_token);

            let now = SystemTime::now();
            let now: DateTime<Utc> = now.into();
            let now = now.to_rfc3339();

            let resp = response.response;

            client
                .put_item()
                .table_name(&config.table_name)
                .item(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
                .item(ACCESS_TOKEN, AttributeValue::S(resp.access_token))
                .item(REFRESH_TOKEN, AttributeValue::S(resp.refresh_token))
                .item(LAST_REFRESH, AttributeValue::S(now))
                .send()
                .await?;
        }

        Some(ref item) => {
            log::info!("{}: tokens in db, trying..", account_name);
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

                    if diff.num_minutes() >= 14 {
                        log::info!("{}: >= 14 mins since last attempt.. refreshing..", account_name);
                        let mut new_access_token = String::from("");
                        let mut new_ref_token = String::from("");

                        let res = api_client.customer_login_refresh(refresh_token).await?;
                        if res.response.is_some() {
                            let unwrapped_res = res.response.unwrap();
                            log::info!("refresh success..");

                            new_access_token = unwrapped_res.access_token;
                            new_ref_token = unwrapped_res.refresh_token;
                        } else if res.status.code != 20000 {
                            let response = api_client.security_auth_token(&config.client_secret).await?;
                            api_client.set_login_token(&response.response.token);

                            let response = api_client.customer_login(login_username, login_password).await?;
                            api_client.set_auth_token(&response.response.access_token);

                            log::info!("refresh failed, logged in again..");
                            new_access_token = response.response.access_token;
                            new_ref_token = response.response.refresh_token;
                        }

                        api_client.set_auth_token(&new_access_token);
                        client
                            .put_item()
                            .table_name(&config.table_name)
                            .item(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
                            .item(ACCESS_TOKEN, AttributeValue::S(new_access_token))
                            .item(REFRESH_TOKEN, AttributeValue::S(new_ref_token))
                            .item(LAST_REFRESH, AttributeValue::S(now.to_rfc3339()))
                            .send()
                            .await?;
                    }
                }
                _ => panic!(),
            };
        }
    }

    Ok(api_client)
}
