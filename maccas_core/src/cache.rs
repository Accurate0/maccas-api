use crate::constants::{ACCOUNT_NAME, LAST_REFRESH, OFFER_LIST};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::Utc;
use lambda_runtime::Error;
use libmaccas::api::ApiClient;
use libmaccas::types::Offer;
use std::collections::HashMap;
use std::time::SystemTime;

pub async fn get_offers<'a>(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &'a String,
    client_map: &'a HashMap<String, ApiClient>,
) -> Result<HashMap<&'a String, Vec<Offer>>, Error> {
    let mut offer_map = HashMap::<&String, Vec<Offer>>::new();
    for (account_name, api_client) in client_map {
        let table_resp = client
            .get_item()
            .table_name(cache_table_name)
            .key(ACCOUNT_NAME, AttributeValue::S(account_name.to_string()))
            .send()
            .await?;

        let resp = match table_resp.item {
            Some(ref item) => match item[LAST_REFRESH].as_s() {
                Ok(s) => {
                    let now = SystemTime::now();
                    let now: DateTime<Utc> = now.into();
                    let now: DateTime<FixedOffset> = DateTime::from(now);

                    let last_refresh = DateTime::parse_from_rfc3339(s).unwrap();

                    let diff = now - last_refresh;

                    if diff.num_minutes() > 59 {
                        println!("{}: update offers cache", account_name);
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
                        resp
                    } else {
                        println!("{}: offers in cache", account_name);
                        match item[OFFER_LIST].as_s() {
                            Ok(s) => serde_json::from_str::<Vec<Offer>>(s).unwrap(),
                            _ => panic!(),
                        }
                    }
                }
                _ => panic!(),
            },

            None => {
                println!("{}: no cached values", account_name);
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
                resp
            }
        };

        offer_map.insert(account_name, resp);
    }
    Ok(offer_map)
}
