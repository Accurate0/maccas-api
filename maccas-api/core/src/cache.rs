use crate::constants::{ACCOUNT_NAME, LAST_REFRESH, OFFER_LIST};
use crate::utils;
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::DateTime;
use chrono::Utc;
use lambda_runtime::Error;
use libmaccas::api::ApiClient;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio_stream::StreamExt;
use types::api::Offer;

#[deprecated]
pub async fn get_offers<'a>(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &'a String,
    account_name_list: &'a Vec<String>,
) -> Result<HashMap<&'a String, Option<Vec<Offer>>>, Error> {
    let mut offer_map = HashMap::<&String, Option<Vec<Offer>>>::new();
    for account_name in account_name_list {
        let resp = get_offer_for(&client, &cache_table_name, &account_name).await?;
        offer_map.insert(account_name, resp);
    }
    Ok(offer_map)
}

pub async fn get_all_offers_as_map(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &String,
) -> Result<HashMap<String, Vec<Offer>>, Error> {
    let mut offer_map = HashMap::<String, Vec<Offer>>::new();

    let table_resp: Result<Vec<_>, _> = client
        .scan()
        .table_name(cache_table_name)
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

pub async fn get_all_offers_as_vec(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &String,
) -> Result<Vec<Offer>, Error> {
    let mut offer_list = Vec::<Offer>::new();

    let table_resp: Result<Vec<_>, _> = client
        .scan()
        .table_name(cache_table_name)
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

pub async fn get_offer_for(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &String,
    account_name: &String,
) -> Result<Option<Vec<Offer>>, Error> {
    let table_resp = client
        .get_item()
        .table_name(cache_table_name)
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

pub async fn set_offer_for(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &String,
    account_name: &String,
    offer_list: &Vec<Offer>,
) -> Result<(), Error> {
    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now = now.to_rfc3339();

    client
        .put_item()
        .table_name(cache_table_name)
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

pub async fn refresh_offer_cache<'a>(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &'a String,
    client_map: &'a HashMap<String, ApiClient<'_>>,
) -> Result<(), Error> {
    for (account_name, api_client) in client_map {
        refresh_offer_cache_for(&client, &cache_table_name, &account_name, &api_client).await?;
        utils::remove_all_from_deal_stack_for(&api_client).await?;
    }

    log::info!(
        "refreshed {} account offer caches..",
        client_map.keys().len()
    );

    Ok(())
}

pub async fn refresh_offer_cache_for(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &String,
    account_name: &String,
    api_client: &ApiClient<'_>,
) -> Result<(), Error> {
    let mut resp = api_client
        .get_offers(None)
        .await?
        .response
        .expect("to have response")
        .offers;

    let now = SystemTime::now();
    let now: DateTime<Utc> = now.into();
    let now = now.to_rfc3339();

    let ignored_offers = vec![30762, 162091, 165964];

    let resp: Vec<types::api::Offer> = resp
        .iter_mut()
        .filter(|offer| !ignored_offers.contains(&offer.offer_proposition_id))
        .map(|offer| types::api::Offer::from(offer.clone()))
        .collect();

    client
        .put_item()
        .table_name(cache_table_name)
        .item(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
        .item(LAST_REFRESH, AttributeValue::S(now))
        .item(
            OFFER_LIST,
            AttributeValue::S(serde_json::to_string(&resp).unwrap()),
        )
        .send()
        .await?;

    log::info!("{}: offer cache refreshed", account_name);
    Ok(())
}

pub async fn get_refresh_time_for_offer_cache(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &String,
) -> Result<String, Error> {
    let table_resp = client
        .scan()
        .limit(1)
        .table_name(cache_table_name)
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
