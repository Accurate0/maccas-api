use crate::constants::{ACCOUNT_NAME, LAST_REFRESH, OFFER_LIST};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::DateTime;
use chrono::Utc;
use lambda_runtime::Error;
use libmaccas::api::ApiClient;
use std::collections::HashMap;
use std::time::SystemTime;
use types::maccas::Offer;
use uuid::Uuid;

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
    client_map: &'a HashMap<String, ApiClient>,
) -> Result<(), Error> {
    for (account_name, api_client) in client_map {
        refresh_offer_cache_for(&client, &cache_table_name, &account_name, &api_client).await?;
    }
    println!(
        "refreshed {} account offer caches..",
        client_map.keys().len()
    );

    Ok(())
}

pub async fn refresh_offer_cache_for(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &String,
    account_name: &String,
    api_client: &ApiClient,
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

    let cached_offers_or_none = get_offer_for(&client, &cache_table_name, &account_name).await?;

    // use existing offer id if possible
    // can rely on proposition because thats kind of offer type id
    // unique in account i think..
    let resp: Vec<&mut Offer> = resp
        .iter_mut()
        .map(|offer| {
            if let Some(cached_offers) = &cached_offers_or_none {
                if let Some(cached_offer) = cached_offers
                    .iter()
                    .find(|co| co.offer_proposition_id == offer.offer_proposition_id)
                {
                    match &cached_offer.deal_uuid {
                        Some(u) => {
                            offer.deal_uuid = Some(u.clone());
                        }
                        None => {
                            offer.deal_uuid = Some(Uuid::new_v4().to_hyphenated().to_string());
                        }
                    }
                } else {
                    offer.deal_uuid = Some(Uuid::new_v4().to_hyphenated().to_string());
                }
            } else {
                offer.deal_uuid = Some(Uuid::new_v4().to_hyphenated().to_string());
            }
            offer
        })
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

    println!("{}: offer cache refreshed", account_name);
    Ok(())
}
