use crate::constants::{ACCOUNT_NAME, LAST_REFRESH, OFFER_LIST};
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
        remove_all_from_deal_stack_for(&api_client).await?;
    }
    log::info!(
        "refreshed {} account offer caches..",
        client_map.keys().len()
    );

    Ok(())
}

pub async fn remove_all_from_deal_stack_for(api_client: &ApiClient<'_>) -> Result<(), Error> {
    // honestly, we don't want failures here, so we'll probably just suppress them...
    log::info!("{}: trying to clean deal stack", api_client.username());
    let deal_stack = api_client.offers_dealstack(None, None).await;
    if let Ok(deal_stack) = deal_stack {
        if let Some(deal_stack) = deal_stack.response {
            if let Some(deal_stack) = deal_stack.deal_stack {
                for deal in deal_stack {
                    log::info!(
                        "{}: removing offer -> {}",
                        api_client.username(),
                        deal.offer_id
                    );
                    api_client
                        .remove_offer_from_offers_dealstack(
                            deal.offer_id,
                            &deal.offer_proposition_id.to_string(),
                            None,
                            None,
                        )
                        .await
                        .ok();
                }
            }
        }
    };
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

    let resp: Vec<types::api::Offer> = resp
        .iter_mut()
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
