use super::DynamoDatabase;
use crate::constants::config::{DEFAULT_REFRESH_TTL_HOURS, MAX_PROXY_COUNT};
use crate::constants::db::{
    ACCESS_TOKEN, ACCOUNT_HASH, ACCOUNT_INFO, ACCOUNT_NAME, ACTION, ACTOR, CURRENT_LIST, DEAL_UUID,
    DEVICE_ID, GROUP, KEY, LAST_REFRESH, LOGIN_PASSWORD, LOGIN_USERNAME, OFFER, OFFER_ID,
    OFFER_LIST, OFFER_NAME, OFFER_PROPOSITION_ID, OPERATION_ID, PASSWORD_HASH, POINT_INFO,
    REFRESH_TOKEN, REGION, ROLE, SALT, TIMESTAMP, TTL, USERNAME, USER_CONFIG, USER_ID, USER_NAME,
    VALUE,
};
use crate::constants::mc_donalds;
use crate::database::r#trait::Database;
use crate::database::types::{
    AuditActionType, OfferDatabase, PointsDatabase, User, UserAccountDatabase, UserOptionsDatabase,
};
use crate::extensions::ApiClientExtensions;
use crate::proxy;
use crate::rng::RNG;
use crate::types::audit::AuditEntry;
use crate::types::config::GeneralConfig;
use crate::types::refresh::RefreshOfferCache;
use anyhow::{bail, Context};
use async_trait::async_trait;
use aws_sdk_dynamodb::primitives::Blob;
use aws_sdk_dynamodb::types::{AttributeValue, AttributeValueUpdate};
use chrono::{DateTime, FixedOffset};
use chrono::{Duration, Utc};
use foundation::hash::get_short_sha1;
use foundation::types::role::UserRole;
use http::StatusCode;
use itertools::Itertools;
use libmaccas::ApiClient;
use rand::distributions::{Alphanumeric, DistString};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashMap;
use std::str::FromStr;
use std::time::SystemTime;
use tokio_stream::StreamExt;

// TODO: fix all scans to use last evaluated key
#[async_trait]
impl Database for DynamoDatabase {
    async fn set_user_tokens(
        &self,
        username: &str,
        auth_token: &str,
        refresh_token: &str,
        ttl: Duration,
    ) -> Result<(), anyhow::Error> {
        let utc: DateTime<Utc> = Utc::now().checked_add_signed(ttl).unwrap();

        self.client
            .put_item()
            .table_name(&self.user_tokens)
            .item(USERNAME, AttributeValue::S(username.to_owned()))
            .item(ACCESS_TOKEN, AttributeValue::S(auth_token.to_owned()))
            .item(REFRESH_TOKEN, AttributeValue::S(refresh_token.to_owned()))
            .item(TTL, AttributeValue::N(utc.timestamp().to_string()))
            .send()
            .await?;

        Ok(())
    }

    async fn get_user_tokens(&self, username: String) -> Result<(String, String), anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.user_tokens)
            .key(USERNAME, AttributeValue::S(username))
            .send()
            .await?;

        let item = response.item.context("must have item")?;
        let access_token = item
            .get(ACCESS_TOKEN)
            .context("must have access token")?
            .as_s()
            .cloned()
            .unwrap();
        let refresh_token = item
            .get(REFRESH_TOKEN)
            .context("must have refresh token")?
            .as_s()
            .cloned()
            .unwrap();

        Ok((access_token, refresh_token))
    }

    async fn get_user_id(&self, username: String) -> Result<String, anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username))
            .send()
            .await?;

        let item = response.item.context("must have item")?;
        let user_id = item
            .get(USER_ID)
            .context("must have user id")?
            .as_s()
            .cloned()
            .unwrap();

        Ok(user_id)
    }

    async fn get_user_role(&self, username: String) -> Result<UserRole, anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username))
            .send()
            .await?;

        let item = response.item.context("must have item")?;
        let role = item
            .get(ROLE)
            .context("must have password")?
            .as_s()
            .cloned()
            .unwrap();

        Ok(serde_json::from_str::<UserRole>(&role)?)
    }

    async fn set_user_role(&self, username: String, role: UserRole) -> Result<(), anyhow::Error> {
        self.client
            .update_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username))
            .attribute_updates(
                ROLE,
                AttributeValueUpdate::builder()
                    .value(AttributeValue::S(serde_json::to_string(&role)?))
                    .build(),
            )
            .send()
            .await?;

        Ok(())
    }

    async fn is_user_exist(&self, username: String) -> Result<bool, anyhow::Error> {
        Ok(self
            .client
            .get_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username))
            .send()
            .await?
            .item
            .is_some())
    }

    async fn get_all_users(&self) -> Result<Vec<User>, anyhow::Error> {
        let users = self
            .client
            .scan()
            .table_name(&self.users)
            .send()
            .await?
            .items()
            .unwrap_or_default()
            .iter()
            .map(|i| -> Result<User, anyhow::Error> {
                Ok(User {
                    id: i
                        .get(USER_ID)
                        .context("missing user id")?
                        .as_s()
                        .map_err(|e| anyhow::Error::msg(format!("{:#?}", e)))?
                        .clone(),
                    username: i
                        .get(USERNAME)
                        .context("missing user name")?
                        .as_s()
                        .map_err(|e| anyhow::Error::msg(format!("{:#?}", e)))?
                        .clone(),
                })
            })
            .filter_map(|r| r.ok())
            .collect();

        Ok(users)
    }

    async fn create_user(
        &self,
        user_id: String,
        username: String,
        password_hash: String,
        salt: Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        log::info!("inserting new user: {user_id} / {username}");

        self.client
            .put_item()
            .table_name(&self.users)
            .item(TIMESTAMP, AttributeValue::S(now))
            .item(USER_ID, AttributeValue::S(user_id))
            .item(USERNAME, AttributeValue::S(username))
            .item(PASSWORD_HASH, AttributeValue::S(password_hash))
            .item(SALT, AttributeValue::B(Blob::new(salt)))
            .send()
            .await?;

        Ok(())
    }

    async fn get_password_hash(&self, username: String) -> Result<String, anyhow::Error> {
        let response = self
            .client
            .get_item()
            .table_name(&self.users)
            .key(USERNAME, AttributeValue::S(username))
            .send()
            .await?;

        let item = response.item.context("must have item")?;
        let password_hash = item
            .get(PASSWORD_HASH)
            .context("must have password")?
            .as_s()
            .cloned()
            .unwrap();

        Ok(password_hash)
    }

    async fn add_to_audit(
        &self,
        action: AuditActionType,
        user_id: Option<String>,
        actor: String,
        offer: &OfferDatabase,
    ) {
        let user_id = user_id.unwrap_or_else(|| "unknown".to_owned());

        log::info!("adding to audit table: {user_id}/{actor} {:?}", offer);

        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();
        let offer_attribute = serde_dynamo::to_item(offer);

        if let Err(e) = offer_attribute {
            log::error!("error adding to audit table: {e}");
            return;
        }

        if let Err(e) = self
            .client
            .put_item()
            .table_name(&self.audit_table_name)
            .item(
                OPERATION_ID,
                AttributeValue::S(uuid::Uuid::now_v7().to_string()),
            )
            .item(ACTION, AttributeValue::S(action.to_string()))
            .item(DEAL_UUID, AttributeValue::S(offer.deal_uuid.to_string()))
            .item(USER_ID, AttributeValue::S(user_id))
            .item(USER_NAME, AttributeValue::S(actor.clone()))
            .item(ACTOR, AttributeValue::S(actor))
            .item(OFFER_NAME, AttributeValue::S(offer.short_name.to_string()))
            .item(TIMESTAMP, AttributeValue::S(now))
            .item(OFFER, AttributeValue::M(offer_attribute.unwrap()))
            .send()
            .await
        {
            log::error!("error adding to audit table: {e}")
        };
    }

    async fn get_audit_entries_for(
        &self,
        user_id: String,
    ) -> Result<Vec<AuditEntry>, anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.audit_table_name)
            .index_name(&self.audit_user_id_index)
            .key_condition_expression("#user = :user_id")
            .expression_attribute_names("#user", USER_ID)
            .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
            .send()
            .await?;

        Ok(resp
            .items()
            .context("no entries for user id provided")
            .unwrap_or_default()
            .iter()
            .map(|item| {
                let action = AuditActionType::from_str(item[ACTION].as_s().unwrap()).unwrap();
                let offer = serde_dynamo::from_item(item[OFFER].as_m().unwrap().clone()).unwrap();
                AuditEntry {
                    action,
                    offer,
                    user_id: user_id.to_string(),
                }
            })
            .collect_vec())
    }

    async fn set_last_refresh(&self) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        self.client
            .put_item()
            .table_name(&self.last_refresh_table_name)
            .item(KEY, AttributeValue::S(LAST_REFRESH.to_owned()))
            .item(VALUE, AttributeValue::S(now))
            .send()
            .await?;

        Ok(())
    }

    async fn get_last_refresh(&self) -> Result<String, anyhow::Error> {
        let item = self
            .client
            .get_item()
            .table_name(&self.last_refresh_table_name)
            .key(KEY, AttributeValue::S(LAST_REFRESH.to_owned()))
            .send()
            .await?;

        match item.item().context("no items") {
            Ok(map) => {
                let value = map.get(VALUE);
                Ok(value
                    .unwrap_or(&AttributeValue::S(
                        Into::<DateTime<Utc>>::into(SystemTime::UNIX_EPOCH).to_rfc3339(),
                    ))
                    .as_s()
                    .unwrap()
                    .to_string())
            }
            Err(_) => Ok(Into::<DateTime<Utc>>::into(SystemTime::UNIX_EPOCH).to_rfc3339()),
        }
    }

    async fn increment_refresh_tracking(
        &self,
        region: &str,
        max_count: i8,
    ) -> Result<i8, anyhow::Error> {
        let table_resp = self
            .client
            .get_item()
            .table_name(&self.refresh_tracking_table_name)
            .key(REGION, AttributeValue::S(region.to_string()))
            .send()
            .await?;

        let item = table_resp.item();

        match item {
            Some(item) => {
                let count = item[CURRENT_LIST].as_n().unwrap();
                let mut new_count = count.parse::<i8>().unwrap() + 1;
                if new_count >= max_count {
                    new_count = 0;
                }

                self.client
                    .put_item()
                    .table_name(&self.refresh_tracking_table_name)
                    .item(REGION, AttributeValue::S(region.to_string()))
                    .item(CURRENT_LIST, AttributeValue::N(new_count.to_string()))
                    .send()
                    .await?;

                Ok(new_count)
            }
            None => {
                self.client
                    .put_item()
                    .table_name(&self.refresh_tracking_table_name)
                    .item(REGION, AttributeValue::S(region.to_string()))
                    .item(CURRENT_LIST, AttributeValue::N("0".to_string()))
                    .send()
                    .await?;
                Ok(0)
            }
        }
    }

    async fn get_all_offers_as_map(
        &self,
    ) -> Result<HashMap<String, Vec<OfferDatabase>>, anyhow::Error> {
        let mut offer_map = HashMap::<String, Vec<OfferDatabase>>::new();

        let table_resp: Result<Vec<_>, _> = self
            .client
            .scan()
            .table_name(&self.account_cache_table_name)
            .into_paginator()
            .items()
            .send()
            .collect()
            .await;

        for item in table_resp? {
            if item[ACCOUNT_NAME].as_s().is_ok() && item[OFFER_LIST].as_s().is_ok() {
                let account_name = item[ACCOUNT_NAME].as_s().unwrap();
                let offer_list = item[OFFER_LIST].as_s().unwrap();
                let offer_list = serde_json::from_str::<Vec<OfferDatabase>>(offer_list).unwrap();

                offer_map.insert(account_name.to_string(), offer_list);
            }
        }

        Ok(offer_map)
    }

    async fn get_all_offers_as_vec(&self) -> Result<Vec<OfferDatabase>, anyhow::Error> {
        let mut offer_list = Vec::<OfferDatabase>::new();

        let table_resp: Result<Vec<_>, _> = self
            .client
            .scan()
            .table_name(&self.account_cache_table_name)
            .into_paginator()
            .items()
            .send()
            .collect()
            .await;

        for item in table_resp? {
            match item[OFFER_LIST].as_s() {
                Ok(s) => {
                    let mut json = serde_json::from_str::<Vec<OfferDatabase>>(s).unwrap();
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
    ) -> Result<Option<Vec<OfferDatabase>>, anyhow::Error> {
        let table_resp = self
            .client
            .get_item()
            .table_name(&self.account_cache_table_name)
            .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .send()
            .await?;

        Ok(match table_resp.item {
            Some(ref item) => match item[OFFER_LIST].as_s() {
                Ok(s) => {
                    let json = serde_json::from_str::<Vec<OfferDatabase>>(s).unwrap();
                    Some(json)
                }
                _ => panic!(),
            },

            None => None,
        })
    }

    async fn set_offers_for(
        &self,
        account: &UserAccountDatabase,
        offer_list: &[OfferDatabase],
    ) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        self.client
            .put_item()
            .table_name(&self.account_cache_table_name)
            .item(
                ACCOUNT_NAME,
                AttributeValue::S(account.account_name.to_string()),
            )
            .item(LAST_REFRESH, AttributeValue::S(now))
            .item(
                OFFER_LIST,
                AttributeValue::S(serde_json::to_string(&offer_list).unwrap()),
            )
            .send()
            .await?;

        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();
        // update the lookup structure too
        let ttl: DateTime<Utc> = Utc::now()
            .checked_add_signed(Duration::hours(DEFAULT_REFRESH_TTL_HOURS))
            .unwrap();
        for offer in offer_list {
            self.client
                .put_item()
                .table_name(&self.offer_cache_table_name)
                .item(DEAL_UUID, AttributeValue::S(offer.deal_uuid.clone()))
                .item(
                    ACCOUNT_INFO,
                    AttributeValue::M(serde_dynamo::to_item(account)?),
                )
                .item(LAST_REFRESH, AttributeValue::S(now.clone()))
                .item(OFFER, AttributeValue::M(serde_dynamo::to_item(offer)?))
                .item(TTL, AttributeValue::N(ttl.timestamp().to_string()))
                .send()
                .await?;
        }

        Ok(())
    }

    async fn refresh_offer_cache(
        &self,
        client_map: &HashMap<UserAccountDatabase, ApiClient>,
        ignored_offer_ids: &[i64],
    ) -> Result<RefreshOfferCache, anyhow::Error> {
        let mut failed_accounts = Vec::new();
        let mut new_offers = Vec::new();

        for (account, api_client) in client_map {
            match self
                .refresh_offer_cache_for(account, api_client, ignored_offer_ids)
                .await
            {
                Ok(mut o) => {
                    new_offers.append(&mut o);
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

        Ok(RefreshOfferCache {
            failed_accounts,
            new_offers,
        })
    }

    async fn refresh_point_cache_for(
        &self,
        account: &UserAccountDatabase,
        api_client: &ApiClient,
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
                        AttributeValue::M(serde_dynamo::to_item(PointsDatabase::from(points))?),
                    )
                    .send()
                    .await?;
                Ok(())
            }
            Err(e) => bail!("could not get points for {} because {}", account, e),
        }
    }

    async fn get_point_map(&self) -> Result<HashMap<String, PointsDatabase>, anyhow::Error> {
        let mut point_map = HashMap::<String, PointsDatabase>::new();

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
    ) -> Result<(UserAccountDatabase, PointsDatabase), anyhow::Error> {
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
            let account: UserAccountDatabase = serde_dynamo::from_item(
                item[ACCOUNT_INFO]
                    .as_m()
                    .ok()
                    .context("no account")?
                    .clone(),
            )?;
            let points: PointsDatabase = serde_dynamo::from_item(
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
            .table_name(&self.token_cache_table_name)
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
            .table_name(&self.token_cache_table_name)
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

    async fn set_access_and_refresh_token_for(
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
            .table_name(&self.token_cache_table_name)
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

    async fn refresh_offer_cache_for(
        &self,
        account: &UserAccountDatabase,
        api_client: &ApiClient,
        ignored_offer_ids: &[i64],
    ) -> Result<Vec<OfferDatabase>, anyhow::Error> {
        log::info!("{}: fetching offers", account.account_name);
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
                let ttl: DateTime<Utc> = Utc::now()
                    .checked_add_signed(Duration::hours(DEFAULT_REFRESH_TTL_HOURS))
                    .unwrap();

                let mut price_map = HashMap::new();
                for offer_proposition_id in resp
                    .iter()
                    .unique_by(|offer| offer.offer_proposition_id)
                    .filter(|offer| !ignored_offer_ids.contains(&offer.offer_proposition_id))
                    .map(|offer| offer.offer_proposition_id)
                {
                    let res = api_client.get_offer_details(offer_proposition_id).await?;
                    if let Some(offer) = res.response {
                        let total_price =
                            offer.product_sets.iter().fold(0f64, |accumulator, item| {
                                if let Some(action) = &item.action {
                                    action.value + accumulator
                                } else {
                                    accumulator
                                }
                            });

                        price_map.insert(offer.offer_proposition_id, total_price);
                    }
                }

                let resp: Vec<OfferDatabase> = resp
                    .iter_mut()
                    .filter(|offer| !ignored_offer_ids.contains(&offer.offer_proposition_id))
                    .map(|offer| OfferDatabase::from(offer.clone()))
                    .map(|mut offer| {
                        let price = price_map.get(&offer.offer_proposition_id).copied();
                        if price != Some(0f64) {
                            offer.price = price;
                        }
                        offer
                    })
                    .collect();

                // v2 cache structure
                for item in &resp {
                    self.client
                        .put_item()
                        .table_name(&self.offer_cache_table_name)
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

                // v3 cache structure
                // find all old entries
                log::info!("finding entries to remove");
                let to_delete = self
                    .client
                    .query()
                    .table_name(&self.current_deals)
                    .index_name(&self.current_deals_account_name)
                    .key_condition_expression("#name = :name")
                    .expression_attribute_names("#name", ACCOUNT_NAME)
                    .expression_attribute_values(
                        ":name",
                        AttributeValue::S(account.account_name.to_string()),
                    )
                    .send()
                    .await?;

                // add new ones immediately
                for item in &resp {
                    self.client
                        .put_item()
                        .table_name(&self.current_deals)
                        .item(DEAL_UUID, AttributeValue::S(item.deal_uuid.clone()))
                        .item(
                            ACCOUNT_NAME,
                            AttributeValue::S(account.account_name.to_owned()),
                        )
                        .item(LAST_REFRESH, AttributeValue::S(now.clone()))
                        .item(
                            OFFER_PROPOSITION_ID,
                            AttributeValue::S(item.offer_proposition_id.to_string()),
                        )
                        .send()
                        .await?;
                }

                // remove old ones
                if let Some(items) = to_delete.items() {
                    log::info!("removing {} entries", items.len());
                    for item in items {
                        let id = item.get(DEAL_UUID);
                        if let Some(id) = id {
                            self.client
                                .delete_item()
                                .table_name(&self.current_deals)
                                .key(DEAL_UUID, id.clone())
                                .send()
                                .await?;
                        }
                    }
                }

                self.client
                    .put_item()
                    .table_name(&self.account_cache_table_name)
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

                log::info!("{}: offer cache refreshed", account);
                Ok(resp)
            }
            None => bail!("could not get offers for {}", account),
        }
    }

    async fn find_all_by_proposition_id(
        &self,
        proposition_id: &str,
    ) -> Result<Vec<String>, anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.current_deals)
            .index_name(&self.current_deals_offer_proposition_id)
            .key_condition_expression("#id = :id")
            .expression_attribute_names("#id", OFFER_PROPOSITION_ID)
            .expression_attribute_values(":id", AttributeValue::S(proposition_id.to_string()))
            .send()
            .await?;

        Ok(resp
            .items()
            .unwrap_or_default()
            .iter()
            .map(|o| o.get(DEAL_UUID).unwrap().as_s().unwrap().to_owned())
            .collect_vec())
    }

    async fn get_offer_by_id(
        &self,
        offer_id: &str,
    ) -> Result<(UserAccountDatabase, OfferDatabase), anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.offer_cache_table_name)
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
        let offer: OfferDatabase =
            serde_dynamo::from_item(resp[OFFER].as_m().ok().context("missing value")?.clone())?;

        Ok((account, offer))
    }

    async fn get_config_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<UserOptionsDatabase, anyhow::Error> {
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
            let config: UserOptionsDatabase = serde_dynamo::from_item(
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
        user_config: &UserOptionsDatabase,
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
            .table_name(&self.token_cache_table_name)
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
                .table_name(&self.token_cache_table_name)
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
                                    .table_name(&self.token_cache_table_name)
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
            .table_name(&self.locked_offers_table_name)
            .item(OFFER_ID, AttributeValue::S(deal_id.to_string()))
            .item(TTL, AttributeValue::N(utc.timestamp().to_string()))
            .send()
            .await?;

        Ok(())
    }

    async fn unlock_deal(&self, deal_id: &str) -> Result<(), anyhow::Error> {
        self.client
            .delete_item()
            .table_name(&self.locked_offers_table_name)
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
            .table_name(&self.locked_offers_table_name)
            .filter_expression("#ttl_key >= :time")
            .expression_attribute_names("#ttl_key", TTL)
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
                .table_name(&self.locked_offers_table_name)
                .key(OFFER_ID, AttributeValue::S(deal))
                .send()
                .await?;
        }
        Ok(())
    }

    async fn add_user_account(
        &self,
        account_name: &str,
        login_username: &str,
        login_password: &str,
        region: &str,
        group: &str,
    ) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        self.client
            .put_item()
            .table_name(&self.user_accounts)
            .item(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .item(
                LOGIN_USERNAME,
                AttributeValue::S(login_username.to_string()),
            )
            .item(
                LOGIN_PASSWORD,
                AttributeValue::S(login_password.to_string()),
            )
            .item(REGION, AttributeValue::S(region.to_string()))
            .item(GROUP, AttributeValue::S(group.to_string()))
            .item(TIMESTAMP, AttributeValue::S(now))
            .send()
            .await?;

        Ok(())
    }

    async fn get_account(&self, account_name: &str) -> Result<UserAccountDatabase, anyhow::Error> {
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

        let m = resp
            .items()
            .context("no account")?
            .first()
            .context("no account")?;

        Ok(UserAccountDatabase {
            account_name: m[ACCOUNT_NAME].as_s().unwrap().to_string(),
            login_username: m[LOGIN_USERNAME].as_s().unwrap().to_string(),
            login_password: m[LOGIN_PASSWORD].as_s().unwrap().to_string(),
            region: m[REGION].as_s().unwrap().to_string(),
            group: m[GROUP].as_s().unwrap().to_string(),
        })
    }

    async fn get_accounts_for_region_and_group(
        &self,
        region: &str,
        group: &str,
    ) -> Result<Vec<UserAccountDatabase>, anyhow::Error> {
        let resp = self
            .client
            .scan()
            .table_name(&self.user_accounts)
            .filter_expression("#region = :selected_region and #group = :selected_group")
            .expression_attribute_names("#region", REGION)
            .expression_attribute_names("#group", GROUP)
            .expression_attribute_values(":selected_region", AttributeValue::S(region.to_string()))
            .expression_attribute_values(":selected_group", AttributeValue::S(group.to_string()))
            .send()
            .await?;

        let items = resp.items();
        Ok(match items {
            Some(items) => items
                .iter()
                .map(|m| UserAccountDatabase {
                    account_name: m[ACCOUNT_NAME].as_s().unwrap().to_string(),
                    login_username: m[LOGIN_USERNAME].as_s().unwrap().to_string(),
                    login_password: m[LOGIN_PASSWORD].as_s().unwrap().to_string(),
                    region: m[REGION].as_s().unwrap().to_string(),
                    group: m[GROUP].as_s().unwrap().to_string(),
                })
                .collect_vec(),
            None => Vec::default(),
        })
    }
}
