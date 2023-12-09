use super::types::UserAccountDatabase;
use crate::{
    constants::{
        config::MAX_PROXY_COUNT,
        db::{
            ACCESS_TOKEN, ACCOUNT_NAME, DEVICE_ID, GROUP, LAST_REFRESH, LOGIN_PASSWORD,
            LOGIN_USERNAME, REFRESH_TOKEN, REGION, TIMESTAMP,
        },
        mc_donalds,
    },
    proxy,
    rng::RNG,
    types::config::{GeneralConfig, Tables},
};
use anyhow::{bail, Context};
use async_recursion::async_recursion;
use aws_sdk_dynamodb::types::{AttributeValue, AttributeValueUpdate};
use chrono::{DateTime, FixedOffset, Utc};
use http::StatusCode;
use itertools::Itertools;
use libmaccas::ApiClient;
use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::StdRng,
    Rng, SeedableRng,
};
use std::{collections::HashMap, time::SystemTime};

#[derive(Clone)]
pub struct AccountRepository {
    client: aws_sdk_dynamodb::Client,
    user_accounts: String,
    token_cache: String,
}

pub struct UserAccountsFilter<'a> {
    pub region: &'a str,
    pub group: &'a str,
}

impl AccountRepository {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables) -> Self {
        Self {
            client,
            user_accounts: tables.user_accounts.clone(),
            token_cache: tables.token_cache.clone(),
        }
    }

    pub async fn add_user_account(
        &self,
        account: &UserAccountDatabase,
    ) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        self.client
            .put_item()
            .table_name(&self.user_accounts)
            .item(
                ACCOUNT_NAME,
                AttributeValue::S(account.account_name.to_string()),
            )
            .item(
                LOGIN_USERNAME,
                AttributeValue::S(account.login_username.to_string()),
            )
            .item(
                LOGIN_PASSWORD,
                AttributeValue::S(account.login_password.to_string()),
            )
            .item(REGION, AttributeValue::S(account.region.to_string()))
            .item(GROUP, AttributeValue::S(account.group.to_string()))
            .item(TIMESTAMP, AttributeValue::S(now))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_user_account(
        &self,
        account_name: &str,
    ) -> Result<UserAccountDatabase, anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.user_accounts)
            .limit(1)
            .key_condition_expression(format!("{ACCOUNT_NAME} = :selected_account"))
            .expression_attribute_values(
                ":selected_account",
                AttributeValue::S(account_name.to_string()),
            )
            .send()
            .await?;

        let m = resp.items().first().context("no account")?;

        Ok(UserAccountDatabase {
            account_name: m[ACCOUNT_NAME].as_s().unwrap().to_string(),
            login_username: m[LOGIN_USERNAME].as_s().unwrap().to_string(),
            login_password: m[LOGIN_PASSWORD].as_s().unwrap().to_string(),
            region: m[REGION].as_s().unwrap().to_string(),
            group: m[GROUP].as_s().unwrap().to_string(),
        })
    }

    pub async fn get_user_accounts(
        &self,
        filter: &UserAccountsFilter<'_>,
    ) -> Result<Vec<UserAccountDatabase>, anyhow::Error> {
        let resp = self
            .client
            .scan()
            .table_name(&self.user_accounts)
            .filter_expression("#region = :selected_region and #group = :selected_group")
            .expression_attribute_names("#region", REGION)
            .expression_attribute_names("#group", GROUP)
            .expression_attribute_values(
                ":selected_region",
                AttributeValue::S(filter.region.to_string()),
            )
            .expression_attribute_values(
                ":selected_group",
                AttributeValue::S(filter.group.to_string()),
            )
            .send()
            .await?;

        Ok(resp
            .items()
            .iter()
            .map(|m| UserAccountDatabase {
                account_name: m[ACCOUNT_NAME].as_s().unwrap().to_string(),
                login_username: m[LOGIN_USERNAME].as_s().unwrap().to_string(),
                login_password: m[LOGIN_PASSWORD].as_s().unwrap().to_string(),
                region: m[REGION].as_s().unwrap().to_string(),
                group: m[GROUP].as_s().unwrap().to_string(),
            })
            .collect_vec())
    }

    #[async_recursion]
    pub async fn get_api_client<'b>(
        &self,
        http_client: reqwest_middleware::ClientWithMiddleware,
        client_id: &'b str,
        client_secret: &'b str,
        sensor_data: &'b str,
        account: &'b UserAccountDatabase,
        force_login: bool,
    ) -> Result<ApiClient, anyhow::Error> {
        let mut api_client = ApiClient::new(
            mc_donalds::default::BASE_URL.to_string(),
            http_client.to_owned(),
            client_id.to_string(),
        );

        let resp = self
            .client
            .get_item()
            .table_name(&self.token_cache)
            .key(
                ACCOUNT_NAME,
                AttributeValue::S(account.account_name.to_string()),
            )
            .send()
            .await?;

        if resp.item.is_none() || force_login {
            log::info!("{}: nothing in db, requesting..", account.account_name);
            if force_login {
                log::info!("{}: login forced", account.account_name);
            }

            let response = api_client.security_auth_token(client_secret).await?;
            api_client.set_login_token(&response.body.response.token);

            let device_id = if force_login && resp.item.is_some() {
                let item = resp.item.unwrap();
                let device_id = item.get(DEVICE_ID);
                match device_id {
                    Some(device_id) => match device_id.as_s() {
                        Ok(s) => s.clone(),
                        _ => bail!("invalid device id for {}", account.account_name),
                    },
                    None => {
                        let mut rng = StdRng::from_entropy();
                        Alphanumeric.sample_string(&mut rng, 16)
                    }
                }
            } else {
                let mut rng = StdRng::from_entropy();
                Alphanumeric.sample_string(&mut rng, 16)
            };

            let response = api_client
                .customer_login(
                    &account.login_username,
                    &account.login_password,
                    sensor_data,
                    &device_id,
                )
                .await?;
            api_client.set_auth_token(&response.body.response.access_token);

            let now = SystemTime::now();
            let now: DateTime<Utc> = now.into();
            let now = now.to_rfc3339();

            let resp = response.body.response;

            self.client
                .put_item()
                .table_name(&self.token_cache)
                .item(
                    ACCOUNT_NAME,
                    AttributeValue::S(account.account_name.to_string()),
                )
                .item(ACCESS_TOKEN, AttributeValue::S(resp.access_token))
                .item(REFRESH_TOKEN, AttributeValue::S(resp.refresh_token))
                .item(LAST_REFRESH, AttributeValue::S(now))
                .item(DEVICE_ID, AttributeValue::S(device_id))
                .send()
                .await?;
        } else {
            match resp.item {
                None => {}
                Some(ref item) => {
                    log::info!("{}: tokens in db, trying..", account.account_name);

                    let device_id = item.get(DEVICE_ID);
                    let device_id = match device_id {
                        Some(device_id) => match device_id.as_s() {
                            Ok(s) => s.clone(),
                            _ => bail!("missing device id for {}", account.account_name),
                        },
                        None => {
                            let mut rng = StdRng::from_entropy();
                            Alphanumeric.sample_string(&mut rng, 16)
                        }
                    };

                    // if missing force a refresh and re-exec
                    let refresh_token = item.get(REFRESH_TOKEN);
                    let refresh_token = match refresh_token {
                        Some(refresh_token) => match refresh_token.as_s() {
                            Ok(s) => s.clone(),
                            _ => bail!("invalid refresh token for {}", account.account_name),
                        },
                        None => {
                            return self
                                .get_api_client(
                                    http_client,
                                    client_id,
                                    client_secret,
                                    sensor_data,
                                    account,
                                    true,
                                )
                                .await;
                        }
                    };

                    let access_token = item.get(ACCESS_TOKEN);
                    let access_token = match access_token {
                        Some(access_token) => match access_token.as_s() {
                            Ok(s) => s.clone(),
                            _ => bail!("invalid access token for {}", account.account_name),
                        },
                        None => {
                            return self
                                .get_api_client(
                                    http_client,
                                    client_id,
                                    client_secret,
                                    sensor_data,
                                    account,
                                    true,
                                )
                                .await;
                        }
                    };

                    api_client.set_auth_token(&access_token);

                    match item[LAST_REFRESH].as_s() {
                        Ok(s) => {
                            let now = SystemTime::now();
                            let now: DateTime<Utc> = now.into();
                            let now: DateTime<FixedOffset> = DateTime::from(now);

                            let last_refresh =
                                DateTime::parse_from_rfc3339(s).context("Invalid date string")?;

                            let diff = now - last_refresh;

                            if diff.num_minutes() >= 14 {
                                log::info!(
                                    "{}: >= 14 mins since last attempt.. refreshing..",
                                    account.account_name
                                );

                                let res = api_client.customer_login_refresh(&refresh_token).await;
                                let (new_access_token, new_ref_token) = if let Ok(res) = res {
                                    // maccas api return 200OK with an error message
                                    if res.status == StatusCode::OK && res.body.status.code == 20000
                                    {
                                        let unwrapped_res =
                                            res.body.response.context("no response")?;
                                        log::info!("refresh success..");

                                        let new_access_token = unwrapped_res.access_token;
                                        let new_ref_token = unwrapped_res.refresh_token;

                                        (new_access_token, new_ref_token)
                                    } else {
                                        let response =
                                            api_client.security_auth_token(client_secret).await?;
                                        api_client.set_login_token(&response.body.response.token);

                                        let response = api_client
                                            .customer_login(
                                                &account.login_username,
                                                &account.login_password,
                                                &sensor_data,
                                                &device_id,
                                            )
                                            .await?;

                                        log::info!("refresh failed, logged in again..");
                                        let new_access_token = response.body.response.access_token;
                                        let new_ref_token = response.body.response.refresh_token;

                                        (new_access_token, new_ref_token)
                                    }
                                } else {
                                    let response =
                                        api_client.security_auth_token(client_secret).await?;
                                    api_client.set_login_token(&response.body.response.token);

                                    let response = api_client
                                        .customer_login(
                                            &account.login_username,
                                            &account.login_password,
                                            &sensor_data,
                                            &device_id,
                                        )
                                        .await?;

                                    log::info!("refresh failed, logged in again..");
                                    let new_access_token = response.body.response.access_token;
                                    let new_ref_token = response.body.response.refresh_token;

                                    (new_access_token, new_ref_token)
                                };

                                api_client.set_auth_token(&new_access_token);
                                self.client
                                    .put_item()
                                    .table_name(&self.token_cache)
                                    .item(
                                        ACCOUNT_NAME,
                                        AttributeValue::S(account.account_name.to_string()),
                                    )
                                    .item(ACCESS_TOKEN, AttributeValue::S(new_access_token))
                                    .item(REFRESH_TOKEN, AttributeValue::S(new_ref_token))
                                    .item(LAST_REFRESH, AttributeValue::S(now.to_rfc3339()))
                                    .item(DEVICE_ID, AttributeValue::S(device_id))
                                    .send()
                                    .await?;
                            }
                        }
                        _ => bail!("missing last refresh time for {}", account.account_name),
                    };
                }
            }
        }

        Ok(api_client)
    }

    pub async fn get_api_clients<'b>(
        &self,
        config: &GeneralConfig,
        client_id: &'b str,
        client_secret: &'b str,
        sensor_data: &'b str,
        account_list: &'b [UserAccountDatabase],
        force_login: bool,
    ) -> Result<(HashMap<UserAccountDatabase, ApiClient>, Vec<String>), anyhow::Error> {
        let mut failed_accounts = Vec::new();
        let mut client_map = HashMap::<UserAccountDatabase, ApiClient>::new();
        let mut rng = RNG.lock().await;
        for user in account_list {
            let random_number = rng.gen_range(1..=MAX_PROXY_COUNT);
            log::info!(
                "[get_client_map] using proxy number: {} for {}",
                random_number,
                user.account_name
            );
            let proxy = proxy::get_specific_proxy(&config.proxy, random_number);
            let http_client = foundation::http::get_default_http_client_with_proxy(proxy);

            match self
                .get_api_client(
                    http_client,
                    client_id,
                    client_secret,
                    sensor_data,
                    user,
                    force_login,
                )
                .await
            {
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

    pub async fn get_device_id_for(
        &self,
        account_name: &str,
    ) -> Result<Option<String>, anyhow::Error> {
        let resp = self
            .client
            .get_item()
            .table_name(&self.token_cache)
            .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .send()
            .await?;

        let item = resp.item;
        if let Some(item) = item {
            match item.get(DEVICE_ID) {
                Some(s) => return Ok(Some(s.as_s().unwrap().clone())),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn set_device_id_for(
        &self,
        account_name: &str,
        device_id: &str,
    ) -> Result<(), anyhow::Error> {
        self.client
            .update_item()
            .table_name(&self.token_cache)
            .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .attribute_updates(
                DEVICE_ID,
                AttributeValueUpdate::builder()
                    .value(AttributeValue::S(device_id.to_string()))
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }

    pub async fn set_access_and_refresh_token_for(
        &self,
        account_name: &str,
        access_token: &str,
        refresh_token: &str,
    ) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        self.client
            .update_item()
            .table_name(&self.token_cache)
            .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .attribute_updates(
                ACCESS_TOKEN,
                AttributeValueUpdate::builder()
                    .value(AttributeValue::S(access_token.to_string()))
                    .build(),
            )
            .attribute_updates(
                REFRESH_TOKEN,
                AttributeValueUpdate::builder()
                    .value(AttributeValue::S(refresh_token.to_string()))
                    .build(),
            )
            .attribute_updates(
                LAST_REFRESH,
                AttributeValueUpdate::builder()
                    .value(AttributeValue::S(now.clone()))
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }
}
