use super::types::{OfferDatabase, UserAccountDatabase};
use crate::{
    constants::{
        config::DEFAULT_REFRESH_TTL_HOURS,
        db::{
            ACCOUNT_INFO, ACCOUNT_NAME, DEAL_UUID, LAST_REFRESH, OFFER, OFFER_ID, OFFER_LIST,
            OFFER_PROPOSITION_ID, TTL,
        },
        mc_donalds,
    },
    extensions::ApiClientExtensions,
    types::config::{Indexes, Tables},
};
use anyhow::{bail, Context};
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use libmaccas::ApiClient;
use std::iter::Iterator;
use std::{collections::HashMap, time::SystemTime};

struct Index {
    current_deals_account_name: String,
    current_deals_proposition_id: String,
}

pub struct OfferRepository {
    client: aws_sdk_dynamodb::Client,
    account_cache: String,
    offer_cache: String,
    current_deals: String,
    locked_offers: String,

    index: Index,
}

impl OfferRepository {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables, indexes: &Indexes) -> Self {
        Self {
            client,
            account_cache: tables.account_cache.clone(),
            offer_cache: tables.deal_cache.clone(),
            current_deals: tables.current_deals.clone(),
            locked_offers: tables.locked_offers.clone(),
            index: Index {
                current_deals_proposition_id: indexes.current_deals_offer_proposition_id.clone(),
                current_deals_account_name: indexes.current_deals_account_name.clone(),
            },
        }
    }

    pub async fn get_all_offers_as_map(
        &self,
    ) -> Result<HashMap<String, Vec<OfferDatabase>>, anyhow::Error> {
        let mut offer_map = HashMap::<String, Vec<OfferDatabase>>::new();

        let table_resp: Result<Vec<_>, _> = self
            .client
            .scan()
            .table_name(&self.account_cache)
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

    pub async fn get_all_offers_as_vec(&self) -> Result<Vec<OfferDatabase>, anyhow::Error> {
        let mut offer_list = Vec::<OfferDatabase>::new();

        let table_resp: Result<Vec<_>, _> = self
            .client
            .scan()
            .table_name(&self.account_cache)
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

    pub async fn get_offers_for(
        &self,
        account_name: &str,
    ) -> Result<Option<Vec<OfferDatabase>>, anyhow::Error> {
        let table_resp = self
            .client
            .get_item()
            .table_name(&self.account_cache)
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

    pub async fn set_offers_for(
        &self,
        account: &UserAccountDatabase,
        offer_list: &[OfferDatabase],
    ) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        self.client
            .put_item()
            .table_name(&self.account_cache)
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
                .table_name(&self.offer_cache)
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

    pub async fn refresh_offer_cache_for(
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
                        .table_name(&self.offer_cache)
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
                    .index_name(&self.index.current_deals_account_name)
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
                log::info!("removing {} entries", to_delete.items().len());
                for item in to_delete.items() {
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

                self.client
                    .put_item()
                    .table_name(&self.account_cache)
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

    pub async fn find_all_by_proposition_id(
        &self,
        proposition_id: &str,
    ) -> Result<Vec<String>, anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.current_deals)
            .index_name(&self.index.current_deals_proposition_id)
            .key_condition_expression("#id = :id")
            .expression_attribute_names("#id", OFFER_PROPOSITION_ID)
            .expression_attribute_values(":id", AttributeValue::S(proposition_id.to_string()))
            .send()
            .await?;

        Ok(resp
            .items()
            .iter()
            .map(|o| o.get(DEAL_UUID).unwrap().as_s().unwrap().to_owned())
            .collect_vec())
    }

    pub async fn get_offer_by_id(
        &self,
        offer_id: &str,
    ) -> Result<(UserAccountDatabase, OfferDatabase), anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.offer_cache)
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

    pub async fn lock_deal(&self, deal_id: &str, duration: Duration) -> Result<(), anyhow::Error> {
        let utc: DateTime<Utc> = Utc::now().checked_add_signed(duration).unwrap();

        self.client
            .put_item()
            .table_name(&self.locked_offers)
            .item(OFFER_ID, AttributeValue::S(deal_id.to_string()))
            .item(TTL, AttributeValue::N(utc.timestamp().to_string()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn unlock_deal(&self, deal_id: &str) -> Result<(), anyhow::Error> {
        self.client
            .delete_item()
            .table_name(&self.locked_offers)
            .key(OFFER_ID, AttributeValue::S(deal_id.to_string()))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_all_locked_deals(&self) -> Result<Vec<String>, anyhow::Error> {
        let mut locked_deal_list = Vec::<String>::new();
        let utc: DateTime<Utc> = Utc::now();
        let resp = self
            .client
            .scan()
            .table_name(&self.locked_offers)
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

    pub async fn delete_all_locked_deals(&self) -> Result<(), anyhow::Error> {
        log::info!("deleting all locked deals");
        let locked_deals = self.get_all_locked_deals().await?;
        for deal in locked_deals {
            self.client
                .delete_item()
                .table_name(&self.locked_offers)
                .key(OFFER_ID, AttributeValue::S(deal))
                .send()
                .await?;
        }
        Ok(())
    }
}
