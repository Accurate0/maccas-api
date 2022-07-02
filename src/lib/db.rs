use crate::config::{ApiConfig, UserAccount};
use crate::constants::db::{
    ACCOUNT_HASH, ACCOUNT_INFO, ACCOUNT_NAME, DEAL_UUID, LAST_REFRESH, OFFER, OFFER_LIST, POINT_INFO, TTL, USER_CONFIG,
    USER_ID, USER_NAME,
};
use crate::constants::mc_donalds;
use crate::types::api::{Offer, PointsResponse};
use crate::types::user::UserOptions;
use crate::utils::{self, get_short_sha1};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::DateTime;
use chrono::{Duration, Utc};
use lambda_http::Error;
use libmaccas::ApiClient;
use std::collections::HashMap;
use std::time::SystemTime;
use tokio_stream::StreamExt;

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

pub async fn refresh_offer_cache(
    client: &aws_sdk_dynamodb::Client,
    config: &ApiConfig,
    client_map: &HashMap<UserAccount, ApiClient<'_>>,
) -> Result<Vec<String>, Error> {
    let mut failed_accounts = Vec::new();

    for (account, api_client) in client_map {
        match refresh_offer_cache_for(
            &client,
            &config.cache_table_name,
            &config.cache_table_name_v2,
            &account,
            &api_client,
        )
        .await
        {
            Ok(_) => {
                utils::remove_all_from_deal_stack_for(&api_client, &account.account_name).await?;
                refresh_point_cache_for(&client, &config.point_table_name, account, api_client).await?;
            }
            Err(e) => {
                log::error!("{}: {}", account, e);
                failed_accounts.push(account.account_name.clone());
            }
        };
    }

    log::info!("refreshed {} account offer caches..", client_map.keys().len());
    Ok(failed_accounts)
}

pub async fn refresh_point_cache_for(
    client: &aws_sdk_dynamodb::Client,
    point_table_name: &String,
    account: &UserAccount,
    api_client: &ApiClient<'_>,
) -> Result<(), Error> {
    match api_client.get_customer_points().await {
        Ok(resp) => {
            let now = SystemTime::now();
            let now: DateTime<Utc> = now.into();
            let now = now.to_rfc3339();

            let points = resp.body.response;
            client
                .put_item()
                .table_name(point_table_name)
                .item(
                    ACCOUNT_HASH,
                    AttributeValue::S(get_short_sha1(&account.account_name.to_string())),
                )
                .item(ACCOUNT_NAME, AttributeValue::S(account.account_name.to_string()))
                .item(ACCOUNT_INFO, AttributeValue::M(serde_dynamo::to_item(account)?))
                .item(LAST_REFRESH, AttributeValue::S(now.clone()))
                .item(
                    POINT_INFO,
                    AttributeValue::M(serde_dynamo::to_item(PointsResponse::from(points))?),
                )
                .send()
                .await?;
            Ok(())
        }
        Err(e) => Err(format!("could not get points for {} because {}", account, e).into()),
    }
}

pub async fn get_point_map(
    client: &aws_sdk_dynamodb::Client,
    point_table_name: &String,
) -> Result<HashMap<String, PointsResponse>, Error> {
    let mut point_map = HashMap::<String, PointsResponse>::new();

    let table_resp: Result<Vec<_>, _> = client
        .scan()
        .table_name(point_table_name)
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

pub async fn get_points_by_account_hash(
    client: &aws_sdk_dynamodb::Client,
    table_name: &String,
    account_hash: &str,
) -> Result<(UserAccount, PointsResponse), Error> {
    let resp = client
        .query()
        .table_name(table_name)
        .key_condition_expression("#hash = :account_hash")
        .expression_attribute_names("#hash", ACCOUNT_HASH)
        .expression_attribute_values(":account_hash", AttributeValue::S(account_hash.to_string()))
        .send()
        .await?;

    if resp.items().ok_or("no user config found")?.len() == 1 {
        let item = resp.items().unwrap().first().unwrap();
        let account: UserAccount = serde_dynamo::from_item(item[ACCOUNT_INFO].as_m().ok().ok_or("no config")?.clone())?;
        let points: PointsResponse = serde_dynamo::from_item(item[POINT_INFO].as_m().ok().ok_or("no config")?.clone())?;

        Ok((account, points))
    } else {
        Err("error fetching user config".into())
    }
}

pub async fn refresh_offer_cache_for(
    client: &aws_sdk_dynamodb::Client,
    cache_table_name: &String,
    cache_table_name_v2: &String,
    account: &UserAccount,
    api_client: &ApiClient<'_>,
) -> Result<(), Error> {
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

            // zh-CHS ???
            let ignored_offers = vec![30762, 162091, 165964, 2946152, 3067279];

            let resp: Vec<Offer> = resp
                .iter_mut()
                .filter(|offer| !ignored_offers.contains(&offer.offer_proposition_id))
                .map(|offer| Offer::from(offer.clone()))
                .collect();

            client
                .put_item()
                .table_name(cache_table_name)
                .item(ACCOUNT_NAME, AttributeValue::S(account.account_name.to_string()))
                .item(LAST_REFRESH, AttributeValue::S(now.clone()))
                .item(OFFER_LIST, AttributeValue::S(serde_json::to_string(&resp).unwrap()))
                .send()
                .await?;

            let ttl: DateTime<Utc> = Utc::now().checked_add_signed(Duration::hours(6)).unwrap();
            // v2 cache structure
            for item in &resp {
                client
                    .put_item()
                    .table_name(cache_table_name_v2)
                    .item(DEAL_UUID, AttributeValue::S(item.deal_uuid.clone()))
                    .item(ACCOUNT_INFO, AttributeValue::M(serde_dynamo::to_item(account)?))
                    .item(LAST_REFRESH, AttributeValue::S(now.clone()))
                    .item(OFFER, AttributeValue::M(serde_dynamo::to_item(item)?))
                    .item(TTL, AttributeValue::N(ttl.timestamp().to_string()))
                    .send()
                    .await?;
            }

            log::info!("{}: offer cache refreshed", account);
            Ok(())
        }
        None => Err(format!("could not get offers for {}", account).into()),
    }
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

pub async fn get_offer_by_id(
    offer_id: &str,
    client: &aws_sdk_dynamodb::Client,
    cache_table_name_v2: &String,
) -> Result<(UserAccount, Offer), Error> {
    let resp = client
        .query()
        .table_name(cache_table_name_v2)
        .key_condition_expression("#uuid = :offer")
        .expression_attribute_names("#uuid", DEAL_UUID)
        .expression_attribute_values(":offer", AttributeValue::S(offer_id.to_string()))
        .send()
        .await?;

    let resp = resp.items.ok_or("missing value")?;
    let resp = resp.first().ok_or("missing value")?;
    let account = serde_dynamo::from_item(resp[ACCOUNT_INFO].as_m().ok().ok_or("missing value")?.clone())?;
    let offer: Offer = serde_dynamo::from_item(resp[OFFER].as_m().ok().ok_or("missing value")?.clone())?;

    Ok((account, offer))
}

pub async fn get_config_by_user_id(
    client: &aws_sdk_dynamodb::Client,
    table_name: &String,
    user_id: &str,
) -> Result<UserOptions, Error> {
    let resp = client
        .query()
        .table_name(table_name)
        .key_condition_expression("#id = :user_id")
        .expression_attribute_names("#id", USER_ID)
        .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
        .send()
        .await?;

    if resp.items().ok_or("no user config found")?.len() == 1 {
        let item = resp.items().unwrap().first().unwrap();
        let config: UserOptions = serde_dynamo::from_item(item[USER_CONFIG].as_m().ok().ok_or("no config")?.clone())?;

        Ok(config)
    } else {
        Err("error fetching user config".into())
    }
}

pub async fn set_config_by_user_id(
    client: &aws_sdk_dynamodb::Client,
    table_name: &String,
    user_id: &str,
    user_config: &UserOptions,
    user_name: &String,
) -> Result<(), Error> {
    client
        .put_item()
        .table_name(table_name)
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
