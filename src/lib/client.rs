use crate::client;
use crate::config::{ApiConfig, UserAccount};
use crate::constants::db::{ACCESS_TOKEN, ACCOUNT_NAME, LAST_REFRESH, REFRESH_TOKEN};
use crate::constants::mc_donalds;
use crate::middleware;
use anyhow::{bail, Context};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, FixedOffset, Utc};
use http::StatusCode;
use libmaccas::ApiClient;
use std::collections::HashMap;
use std::time::Duration;
use std::time::SystemTime;

pub fn get_http_client() -> reqwest_middleware::ClientWithMiddleware {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
        .context("Failed to build http client")
        .unwrap();
    middleware::get_middleware_http_client(client)
}

pub fn get_http_client_with_headers(headers: http::HeaderMap) -> reqwest_middleware::ClientWithMiddleware {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .build()
        .context("Failed to build http client")
        .unwrap();
    middleware::get_middleware_http_client(client)
}

pub async fn get_client_map<'a>(
    http_client: &'a reqwest_middleware::ClientWithMiddleware,
    config: &'a ApiConfig,
    account_list: &'a Vec<UserAccount>,
    client: &'a aws_sdk_dynamodb::Client,
) -> Result<(HashMap<UserAccount, ApiClient<'a>>, Vec<String>), anyhow::Error> {
    let mut failed_accounts = Vec::new();
    let mut client_map = HashMap::<UserAccount, ApiClient<'_>>::new();
    for user in account_list {
        match client::get(http_client, &client, &config, &user).await {
            Ok(c) => {
                client_map.insert(user.clone(), c);
            }
            Err(e) => {
                failed_accounts.push(user.account_name.clone());
                log::error!("could not login into {} because {}", user, e);
            }
        };
    }

    Ok((client_map, failed_accounts))
}

pub async fn get<'a>(
    http_client: &'a reqwest_middleware::ClientWithMiddleware,
    client: &'a aws_sdk_dynamodb::Client,
    config: &'a ApiConfig,
    account: &'a UserAccount,
) -> Result<ApiClient<'a>, anyhow::Error> {
    let mut api_client = ApiClient::new(
        mc_donalds::default::BASE_URL.to_string(),
        http_client,
        config.client_id.clone(),
    );

    let resp = client
        .get_item()
        .table_name(&config.table_name)
        .key(ACCOUNT_NAME, AttributeValue::S(account.account_name.to_string()))
        .send()
        .await?;

    match resp.item {
        None => {
            log::info!("{}: nothing in db, requesting..", account.account_name);

            let response = api_client.security_auth_token(&config.client_secret).await?;
            api_client.set_login_token(&response.body.response.token);

            let response = api_client
                .customer_login(&account.login_username, &account.login_password, &config.sensor_data)
                .await?;
            api_client.set_auth_token(&response.body.response.access_token);

            let now = SystemTime::now();
            let now: DateTime<Utc> = now.into();
            let now = now.to_rfc3339();

            let resp = response.body.response;

            client
                .put_item()
                .table_name(&config.table_name)
                .item(ACCOUNT_NAME, AttributeValue::S(account.account_name.to_string()))
                .item(ACCESS_TOKEN, AttributeValue::S(resp.access_token))
                .item(REFRESH_TOKEN, AttributeValue::S(resp.refresh_token))
                .item(LAST_REFRESH, AttributeValue::S(now))
                .send()
                .await?;
        }

        Some(ref item) => {
            log::info!("{}: tokens in db, trying..", account.account_name);
            let refresh_token = match item[REFRESH_TOKEN].as_s() {
                Ok(s) => s,
                _ => bail!("missing refresh token for {}", account.account_name),
            };

            match item[ACCESS_TOKEN].as_s() {
                Ok(s) => api_client.set_auth_token(s),
                _ => bail!("missing access token for {}", account.account_name),
            };

            match item[LAST_REFRESH].as_s() {
                Ok(s) => {
                    let now = SystemTime::now();
                    let now: DateTime<Utc> = now.into();
                    let now: DateTime<FixedOffset> = DateTime::from(now);

                    let last_refresh = DateTime::parse_from_rfc3339(s).context("Invalid date string")?;

                    let diff = now - last_refresh;

                    if diff.num_minutes() >= 14 {
                        log::info!("{}: >= 14 mins since last attempt.. refreshing..", account.account_name);

                        let res = api_client.customer_login_refresh(refresh_token).await?;
                        let (new_access_token, new_ref_token) = if res.status == StatusCode::OK {
                            let unwrapped_res = res.body.response.unwrap();
                            log::info!("refresh success..");

                            let new_access_token = unwrapped_res.access_token;
                            let new_ref_token = unwrapped_res.refresh_token;

                            (new_access_token, new_ref_token)
                        } else {
                            let response = api_client.security_auth_token(&config.client_secret).await?;
                            api_client.set_login_token(&response.body.response.token);

                            let response = api_client
                                .customer_login(&account.login_username, &account.login_password, &config.sensor_data)
                                .await?;

                            log::info!("refresh failed, logged in again..");
                            let new_access_token = response.body.response.access_token;
                            let new_ref_token = response.body.response.refresh_token;

                            (new_access_token, new_ref_token)
                        };

                        api_client.set_auth_token(&new_access_token);
                        client
                            .put_item()
                            .table_name(&config.table_name)
                            .item(ACCOUNT_NAME, AttributeValue::S(account.account_name.to_string()))
                            .item(ACCESS_TOKEN, AttributeValue::S(new_access_token))
                            .item(REFRESH_TOKEN, AttributeValue::S(new_ref_token))
                            .item(LAST_REFRESH, AttributeValue::S(now.to_rfc3339()))
                            .send()
                            .await?;
                    }
                }
                _ => bail!("missing last refresh time for {}", account.account_name),
            };
        }
    }

    Ok(api_client)
}
