use crate::config::{Tables, UserAccount};
use crate::constants::db::{
    ACCESS_TOKEN, ACCOUNT_HASH, ACCOUNT_INFO, ACCOUNT_NAME, DEAL_UUID, DEVICE_ID, LAST_REFRESH,
    OFFER, OFFER_ID, OFFER_LIST, POINT_INFO, REFRESH_TOKEN, TTL, USER_CONFIG, USER_ID, USER_NAME,
};
use crate::constants::mc_donalds;
use crate::types::api::{Offer, PointsResponse};
use crate::types::user::UserOptions;
use crate::utils::{self, get_short_sha1};
use anyhow::{bail, Context};
use async_trait::async_trait;
use aws_sdk_dynamodb::model::{AttributeValue, AttributeValueUpdate};
use chrono::{DateTime, FixedOffset};
use chrono::{Duration, Utc};
use http::StatusCode;
use libmaccas::ApiClient;
use rand::distributions::{Alphanumeric, DistString};
use rand::prelude::StdRng;
use rand::SeedableRng;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio_stream::StreamExt;

#[async_trait]
pub trait Database {
    async fn get_all_offers_as_map(&self) -> Result<HashMap<String, Vec<Offer>>, anyhow::Error>;
    async fn get_all_offers_as_vec(&self) -> Result<Vec<Offer>, anyhow::Error>;
    async fn get_offers_for(&self, account_name: &str)
        -> Result<Option<Vec<Offer>>, anyhow::Error>;
    async fn set_offers_for(
        &self,
        account_name: &str,
        offer_list: &[Offer],
    ) -> Result<(), anyhow::Error>;
    async fn refresh_offer_cache(
        &self,
        client_map: &HashMap<UserAccount, ApiClient<'_>>,
        ignored_offer_ids: &[i64],
    ) -> Result<Vec<String>, anyhow::Error>;
    async fn refresh_point_cache_for(
        &self,
        account: &UserAccount,
        api_client: &ApiClient<'_>,
    ) -> Result<(), anyhow::Error>;
    async fn get_point_map(&self) -> Result<HashMap<String, PointsResponse>, anyhow::Error>;
    async fn get_points_by_account_hash(
        &self,
        account_hash: &str,
    ) -> Result<(UserAccount, PointsResponse), anyhow::Error>;
    async fn refresh_offer_cache_for(
        &self,
        account: &UserAccount,
        api_client: &ApiClient<'_>,
        ignored_offer_ids: &[i64],
    ) -> Result<Vec<Offer>, anyhow::Error>;
    async fn get_refresh_time_for_offer_cache(&self) -> Result<String, anyhow::Error>;
    async fn get_offer_by_id(&self, offer_id: &str) -> Result<(UserAccount, Offer), anyhow::Error>;
    async fn get_config_by_user_id(&self, user_id: &str) -> Result<UserOptions, anyhow::Error>;
    async fn set_config_by_user_id(
        &self,
        user_id: &str,
        user_config: &UserOptions,
        user_name: &str,
    ) -> Result<(), anyhow::Error>;
    async fn get_specific_client<'a>(
        &self,
        http_client: &'a reqwest_middleware::ClientWithMiddleware,
        client_id: &'a str,
        client_secret: &'a str,
        sensor_data: &'a str,
        account: &'a UserAccount,
        force_login: bool,
    ) -> Result<ApiClient<'a>, anyhow::Error>;
    async fn get_client_map<'a>(
        &self,
        http_client: &'a reqwest_middleware::ClientWithMiddleware,
        client_id: &'a str,
        client_secret: &'a str,
        sensor_data: &'a str,
        account_list: &'a [UserAccount],
        force_login: bool,
    ) -> Result<(HashMap<UserAccount, ApiClient<'a>>, Vec<String>), anyhow::Error>;
    async fn lock_deal(&self, deal_id: &str, duration: Duration) -> Result<(), anyhow::Error>;
    async fn unlock_deal(&self, deal_id: &str) -> Result<(), anyhow::Error>;
    async fn get_all_locked_deals(&self) -> Result<Vec<String>, anyhow::Error>;
    async fn delete_all_locked_deals(&self) -> Result<(), anyhow::Error>;
    async fn get_device_id_for(&self, account_name: &str) -> Result<Option<String>, anyhow::Error>;
    async fn set_device_id_for(
        &self,
        account_name: &str,
        device_id: &str,
    ) -> Result<(), anyhow::Error>;
}

pub struct DynamoDatabase {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
    user_config_table_name: String,
    cache_table_name: String,
    cache_table_name_v2: String,
    offer_id_table_name: String,
    point_table_name: String,
}

impl DynamoDatabase {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables) -> Self {
        Self {
            client,
            table_name: tables.token_cache.to_owned(),
            user_config_table_name: tables.user_config.to_owned(),
            cache_table_name: tables.offer_cache.to_owned(),
            cache_table_name_v2: tables.offer_cache_v2.to_owned(),
            offer_id_table_name: tables.offer_id.to_owned(),
            point_table_name: tables.points.to_owned(),
        }
    }
}

#[async_trait]
impl Database for DynamoDatabase {
    async fn get_all_offers_as_map(&self) -> Result<HashMap<String, Vec<Offer>>, anyhow::Error> {
        let mut offer_map = HashMap::<String, Vec<Offer>>::new();

        let table_resp: Result<Vec<_>, _> = self
            .client
            .scan()
            .table_name(&self.cache_table_name)
            .into_paginator()
            .items()
            .send()
            .collect()
            .await;

        for item in table_resp? {
            if item[ACCOUNT_NAME].as_s().is_ok() && item[OFFER_LIST].as_s().is_ok() {
                let account_name = item[ACCOUNT_NAME].as_s().unwrap();
                let offer_list = item[OFFER_LIST].as_s().unwrap();
                let offer_list = serde_json::from_str::<Vec<Offer>>(offer_list).unwrap();

                offer_map.insert(account_name.to_string(), offer_list);
            }
        }

        Ok(offer_map)
    }

    async fn get_all_offers_as_vec(&self) -> Result<Vec<Offer>, anyhow::Error> {
        let mut offer_list = Vec::<Offer>::new();

        let table_resp: Result<Vec<_>, _> = self
            .client
            .scan()
            .table_name(&self.cache_table_name)
            .into_paginator()
            .items()
            .send()
            .collect()
            .await;

        for item in table_resp? {
            match item[OFFER_LIST].as_s() {
                Ok(s) => {
                    let mut json = serde_json::from_str::<Vec<Offer>>(s).unwrap();
                    offer_list.append(&mut json);
                }
                _ => panic!(),
            }
        }

        Ok(offer_list)
    }

    async fn get_offers_for(
        &self,
        account_name: &str,
    ) -> Result<Option<Vec<Offer>>, anyhow::Error> {
        let table_resp = self
            .client
            .get_item()
            .table_name(&self.cache_table_name)
            .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .send()
            .await?;

        Ok(match table_resp.item {
            Some(ref item) => match item[OFFER_LIST].as_s() {
                Ok(s) => {
                    let json = serde_json::from_str::<Vec<Offer>>(s).unwrap();
                    Some(json)
                }
                _ => panic!(),
            },

            None => None,
        })
    }

    async fn set_offers_for(
        &self,
        account_name: &str,
        offer_list: &[Offer],
    ) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        self.client
            .put_item()
            .table_name(&self.cache_table_name)
            .item(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .item(LAST_REFRESH, AttributeValue::S(now))
            .item(
                OFFER_LIST,
                AttributeValue::S(serde_json::to_string(&offer_list).unwrap()),
            )
            .send()
            .await?;

        Ok(())
    }

    async fn refresh_offer_cache(
        &self,
        client_map: &HashMap<UserAccount, ApiClient<'_>>,
        ignored_offer_ids: &[i64],
    ) -> Result<Vec<String>, anyhow::Error> {
        let mut failed_accounts = Vec::new();

        for (account, api_client) in client_map {
            match self
                .refresh_offer_cache_for(account, api_client, ignored_offer_ids)
                .await
            {
                Ok(_) => {
                    utils::remove_all_from_deal_stack_for(api_client, &account.account_name)
                        .await?;
                    self.refresh_point_cache_for(account, api_client).await?;
                }
                Err(e) => {
                    log::error!("{}: {}", account, e);
                    failed_accounts.push(account.account_name.clone());
                }
            };
        }

        log::info!(
            "refreshed {} account offer caches..",
            client_map.keys().len()
        );
        Ok(failed_accounts)
    }

    async fn refresh_point_cache_for(
        &self,
        account: &UserAccount,
        api_client: &ApiClient<'_>,
    ) -> Result<(), anyhow::Error> {
        match api_client.get_customer_points().await {
            Ok(resp) => {
                let now = SystemTime::now();
                let now: DateTime<Utc> = now.into();
                let now = now.to_rfc3339();

                let points = resp.body.response;
                self.client
                    .put_item()
                    .table_name(&self.point_table_name)
                    .item(
                        ACCOUNT_HASH,
                        AttributeValue::S(get_short_sha1(&account.account_name.to_string())),
                    )
                    .item(
                        ACCOUNT_NAME,
                        AttributeValue::S(account.account_name.to_string()),
                    )
                    .item(
                        ACCOUNT_INFO,
                        AttributeValue::M(serde_dynamo::to_item(account)?),
                    )
                    .item(LAST_REFRESH, AttributeValue::S(now.clone()))
                    .item(
                        POINT_INFO,
                        AttributeValue::M(serde_dynamo::to_item(PointsResponse::from(points))?),
                    )
                    .send()
                    .await?;
                Ok(())
            }
            Err(e) => bail!("could not get points for {} because {}", account, e),
        }
    }

    async fn get_point_map(&self) -> Result<HashMap<String, PointsResponse>, anyhow::Error> {
        let mut point_map = HashMap::<String, PointsResponse>::new();

        let table_resp: Result<Vec<_>, _> = self
            .client
            .scan()
            .table_name(&self.point_table_name)
            .into_paginator()
            .items()
            .send()
            .collect()
            .await;

        for item in table_resp? {
            if item[ACCOUNT_HASH].as_s().is_ok() && item[POINT_INFO].as_m().is_ok() {
                let account_hash = item[ACCOUNT_HASH].as_s().unwrap();
                let points = item[POINT_INFO].as_m().unwrap();
                let points = serde_dynamo::from_item(points.clone()).unwrap();

                point_map.insert(account_hash.to_string(), points);
            }
        }

        Ok(point_map)
    }

    async fn get_points_by_account_hash(
        &self,
        account_hash: &str,
    ) -> Result<(UserAccount, PointsResponse), anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.point_table_name)
            .key_condition_expression("#hash = :account_hash")
            .expression_attribute_names("#hash", ACCOUNT_HASH)
            .expression_attribute_values(
                ":account_hash",
                AttributeValue::S(account_hash.to_string()),
            )
            .send()
            .await?;

        if resp.items().context("no account found")?.len() == 1 {
            let item = resp.items().unwrap().first().unwrap();
            let account: UserAccount = serde_dynamo::from_item(
                item[ACCOUNT_INFO]
                    .as_m()
                    .ok()
                    .context("no account")?
                    .clone(),
            )?;
            let points: PointsResponse = serde_dynamo::from_item(
                item[POINT_INFO].as_m().ok().context("no points")?.clone(),
            )?;

            Ok((account, points))
        } else {
            bail!("error getting account information")
        }
    }

    async fn get_device_id_for(&self, account_name: &str) -> Result<Option<String>, anyhow::Error> {
        let resp = self
            .client
            .get_item()
            .table_name(&self.table_name)
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
            return Ok(None);
        }
    }

    async fn set_device_id_for(
        &self,
        account_name: &str,
        device_id: &str,
    ) -> Result<(), anyhow::Error> {
        self.client
            .update_item()
            .table_name(&self.table_name)
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
    async fn refresh_offer_cache_for(
        &self,
        account: &UserAccount,
        api_client: &ApiClient<'_>,
        ignored_offer_ids: &[i64],
    ) -> Result<Vec<Offer>, anyhow::Error> {
        match api_client
            .get_offers(
                mc_donalds::default::DISTANCE,
                mc_donalds::default::LATITUDE,
                mc_donalds::default::LONGITUDE,
                "",
                mc_donalds::default::OFFSET,
            )
            .await?
            .body
            .response
        {
            Some(resp) => {
                let mut resp = resp.offers;

                let now = SystemTime::now();
                let now: DateTime<Utc> = now.into();
                let now = now.to_rfc3339();

                // keep older deals around for 12hrs
                // hotlinking etc
                let ttl: DateTime<Utc> =
                    Utc::now().checked_add_signed(Duration::hours(12)).unwrap();

                let resp: Vec<Offer> = resp
                    .iter_mut()
                    .filter(|offer| !ignored_offer_ids.contains(&offer.offer_proposition_id))
                    .map(|offer| Offer::from(offer.clone()))
                    .collect();

                self.client
                    .put_item()
                    .table_name(&self.cache_table_name)
                    .item(
                        ACCOUNT_NAME,
                        AttributeValue::S(account.account_name.to_string()),
                    )
                    .item(LAST_REFRESH, AttributeValue::S(now.clone()))
                    .item(
                        OFFER_LIST,
                        AttributeValue::S(serde_json::to_string(&resp).unwrap()),
                    )
                    .item(TTL, AttributeValue::N(ttl.timestamp().to_string()))
                    .send()
                    .await?;

                // v2 cache structure
                for item in &resp {
                    self.client
                        .put_item()
                        .table_name(&self.cache_table_name_v2)
                        .item(DEAL_UUID, AttributeValue::S(item.deal_uuid.clone()))
                        .item(
                            ACCOUNT_INFO,
                            AttributeValue::M(serde_dynamo::to_item(account)?),
                        )
                        .item(LAST_REFRESH, AttributeValue::S(now.clone()))
                        .item(OFFER, AttributeValue::M(serde_dynamo::to_item(item)?))
                        .item(TTL, AttributeValue::N(ttl.timestamp().to_string()))
                        .send()
                        .await?;
                }

                log::info!("{}: offer cache refreshed", account);
                Ok(resp)
            }
            None => bail!("could not get offers for {}", account),
        }
    }

    async fn get_refresh_time_for_offer_cache(&self) -> Result<String, anyhow::Error> {
        let table_resp = self
            .client
            .scan()
            .limit(1)
            .table_name(&self.cache_table_name)
            .send()
            .await
            .unwrap();

        if table_resp.count == 1 {
            Ok(table_resp.items.unwrap().first().unwrap()[LAST_REFRESH]
                .as_s()
                .ok()
                .unwrap()
                .to_string())
        } else {
            panic!()
        }
    }

    async fn get_offer_by_id(&self, offer_id: &str) -> Result<(UserAccount, Offer), anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.cache_table_name_v2)
            .key_condition_expression("#uuid = :offer")
            .expression_attribute_names("#uuid", DEAL_UUID)
            .expression_attribute_values(":offer", AttributeValue::S(offer_id.to_string()))
            .send()
            .await?;

        let resp = resp.items.context("missing value")?;
        let resp = resp.first().context("missing value")?;
        let account = serde_dynamo::from_item(
            resp[ACCOUNT_INFO]
                .as_m()
                .ok()
                .context("missing value")?
                .clone(),
        )?;
        let offer: Offer =
            serde_dynamo::from_item(resp[OFFER].as_m().ok().context("missing value")?.clone())?;

        Ok((account, offer))
    }

    async fn get_config_by_user_id(&self, user_id: &str) -> Result<UserOptions, anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.user_config_table_name)
            .key_condition_expression("#id = :user_id")
            .expression_attribute_names("#id", USER_ID)
            .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
            .send()
            .await?;

        if resp.items().context("no user config found")?.len() == 1 {
            let item = resp.items().unwrap().first().unwrap();
            let config: UserOptions = serde_dynamo::from_item(
                item[USER_CONFIG].as_m().ok().context("no config")?.clone(),
            )?;

            Ok(config)
        } else {
            bail!("error fetching user config for {}", user_id)
        }
    }

    async fn set_config_by_user_id(
        &self,
        user_id: &str,
        user_config: &UserOptions,
        user_name: &str,
    ) -> Result<(), anyhow::Error> {
        self.client
            .put_item()
            .table_name(&self.user_config_table_name)
            .item(USER_ID, AttributeValue::S(user_id.to_string()))
            .item(
                USER_CONFIG,
                AttributeValue::M(serde_dynamo::to_item(user_config).unwrap()),
            )
            .item(USER_NAME, AttributeValue::S(user_name.to_string()))
            .send()
            .await?;

        Ok(())
    }

    async fn get_specific_client<'b>(
        &self,
        http_client: &'b reqwest_middleware::ClientWithMiddleware,
        client_id: &'b str,
        client_secret: &'b str,
        sensor_data: &'b str,
        account: &'b UserAccount,
        force_login: bool,
    ) -> Result<ApiClient<'b>, anyhow::Error> {
        let mut api_client = ApiClient::new(
            mc_donalds::default::BASE_URL.to_string(),
            http_client,
            client_id.to_string(),
        );

        let resp = self
            .client
            .get_item()
            .table_name(&self.table_name)
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
                .table_name(&self.table_name)
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
                            _ => bail!("missing refresh token for {}", account.account_name),
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
                                .get_specific_client(
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
                                .get_specific_client(
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
                                    if res.status == StatusCode::OK {
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
                                    .table_name(&self.table_name)
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

    async fn get_client_map<'b>(
        &self,
        http_client: &'b reqwest_middleware::ClientWithMiddleware,
        client_id: &'b str,
        client_secret: &'b str,
        sensor_data: &'b str,
        account_list: &'b [UserAccount],
        force_login: bool,
    ) -> Result<(HashMap<UserAccount, ApiClient<'b>>, Vec<String>), anyhow::Error> {
        let mut failed_accounts = Vec::new();
        let mut client_map = HashMap::<UserAccount, ApiClient<'_>>::new();
        for user in account_list {
            match self
                .get_specific_client(
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

    async fn lock_deal(&self, deal_id: &str, duration: Duration) -> Result<(), anyhow::Error> {
        let utc: DateTime<Utc> = Utc::now().checked_add_signed(duration).unwrap();

        self.client
            .put_item()
            .table_name(&self.offer_id_table_name)
            .item(OFFER_ID, AttributeValue::S(deal_id.to_string()))
            .item(TTL, AttributeValue::N(utc.timestamp().to_string()))
            .send()
            .await?;

        Ok(())
    }

    async fn unlock_deal(&self, deal_id: &str) -> Result<(), anyhow::Error> {
        self.client
            .delete_item()
            .table_name(&self.offer_id_table_name)
            .key(OFFER_ID, AttributeValue::S(deal_id.to_string()))
            .send()
            .await?;

        Ok(())
    }

    async fn get_all_locked_deals(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut locked_deal_list = Vec::<String>::new();
        let utc: DateTime<Utc> = Utc::now();
        let resp = self
            .client
            .scan()
            .table_name(&self.offer_id_table_name)
            .filter_expression("#ttl_key >= :time")
            .expression_attribute_names("#ttl_key", "ttl")
            .expression_attribute_values(":time", AttributeValue::N(utc.timestamp().to_string()))
            .send()
            .await?;

        match resp.items {
            Some(ref items) => {
                for item in items {
                    match item[OFFER_ID].as_s() {
                        Ok(s) => locked_deal_list.push(s.to_string()),
                        _ => panic!(),
                    }
                }
                Ok(locked_deal_list)
            }
            None => Ok(locked_deal_list),
        }
    }

    async fn delete_all_locked_deals(&self) -> Result<(), anyhow::Error> {
        log::info!("deleting all locked deals");
        let locked_deals = self.get_all_locked_deals().await?;
        for deal in locked_deals {
            self.client
                .delete_item()
                .table_name(&self.offer_id_table_name)
                .key(OFFER_ID, AttributeValue::S(deal))
                .send()
                .await?;
        }
        Ok(())
    }
}
