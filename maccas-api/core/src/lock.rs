use crate::constants::{OFFER_ID, TTL};
use aws_sdk_dynamodb::model::AttributeValue;
use chrono::{DateTime, Duration, Utc};
use lambda_http::Error;

pub async fn lock_deal(
    client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    deal_id: &str,
) -> Result<(), Error> {
    let utc: DateTime<Utc> = Utc::now()
        .checked_add_signed(Duration::minutes(15))
        .unwrap();

    client
        .put_item()
        .table_name(table_name)
        .item(OFFER_ID, AttributeValue::S(deal_id.to_string()))
        .item(TTL, AttributeValue::N(utc.timestamp().to_string()))
        .send()
        .await?;

    Ok(())
}

pub async fn unlock_deal(
    client: &aws_sdk_dynamodb::Client,
    table_name: &str,
    deal_id: &str,
) -> Result<(), Error> {
    client
        .delete_item()
        .table_name(table_name)
        .key(OFFER_ID, AttributeValue::S(deal_id.to_string()))
        .send()
        .await?;

    Ok(())
}

pub async fn get_all_locked_deals(
    client: &aws_sdk_dynamodb::Client,
    table_name: &str,
) -> Result<Vec<String>, Error> {
    let mut locked_deal_list = Vec::<String>::new();
    let utc: DateTime<Utc> = Utc::now();
    let resp = client
        .scan()
        .table_name(table_name)
        .filter_expression("#ttl_key >= :time")
        .expression_attribute_names("#ttl_key", "ttl")
        .expression_attribute_values(":time", AttributeValue::N(utc.timestamp().to_string()))
        .send()
        .await?;

    dbg!(&resp);

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
