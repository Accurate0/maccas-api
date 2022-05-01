use crate::constants::{ACCOUNT_NAME, LAST_REFRESH, OFFER_LIST};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::DateTime;
use chrono::Utc;
use lambda_runtime::Error;
use libmaccas::api::ApiClient;
use std::collections::HashMap;
use std::time::SystemTime;
use types::maccas::Offer;

pub async fn get_offers<'a>(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &'a String,
    account_name_list: &'a Vec<String>,
) -> Result<HashMap<&'a String, Option<Vec<Offer>>>, Error> {
    let mut offer_map = HashMap::<&String, Option<Vec<Offer>>>::new();
    for account_name in account_name_list {
        let table_resp = client
            .get_item()
            .table_name(cache_table_name)
            .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .send()
            .await?;

        let resp = match table_resp.item {
            Some(ref item) => match item[OFFER_LIST].as_s() {
                Ok(s) => {
                    let json = serde_json::from_str::<Vec<Offer>>(s).unwrap();
                    Some(json)
                }
                _ => panic!(),
            },

            None => None,
        };

        offer_map.insert(account_name, resp);
    }
    Ok(offer_map)
}

pub async fn refresh_offer_cache<'a>(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &'a String,
    client_map: &'a HashMap<String, ApiClient>,
) -> Result<(), Error> {
    for (account_name, api_client) in client_map {
        let resp = api_client
            .get_offers(None)
            .await?
            .response
            .expect("to have response")
            .offers;

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
                AttributeValue::S(serde_json::to_string(&resp).unwrap()),
            )
            .send()
            .await?;

        println!("{}: offer cache refreshed", account_name)
    }

    println!(
        "refreshed {} account offer caches..",
        client_map.keys().len()
    );

    Ok(())
}
