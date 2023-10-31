use crate::{
    constants::db::{CURRENT_LIST, KEY, LAST_REFRESH, REGION, VALUE},
    types::config::Tables,
};
use anyhow::Context;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use std::time::SystemTime;

pub struct RefreshRepository {
    client: aws_sdk_dynamodb::Client,
    refresh_tracking: String,
    last_refresh: String,
}

impl RefreshRepository {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables) -> Self {
        Self {
            client,
            refresh_tracking: tables.refresh_tracking.clone(),
            last_refresh: tables.last_refresh.clone(),
        }
    }

    pub async fn set_last_refresh(&self) -> Result<(), anyhow::Error> {
        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();

        self.client
            .put_item()
            .table_name(&self.last_refresh)
            .item(KEY, AttributeValue::S(LAST_REFRESH.to_owned()))
            .item(VALUE, AttributeValue::S(now))
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_last_refresh(&self) -> Result<String, anyhow::Error> {
        let item = self
            .client
            .get_item()
            .table_name(&self.last_refresh)
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

    pub async fn increment_refresh_tracking(
        &self,
        region: &str,
        max_count: i8,
    ) -> Result<i8, anyhow::Error> {
        let table_resp = self
            .client
            .get_item()
            .table_name(&self.refresh_tracking)
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
                    .table_name(&self.refresh_tracking)
                    .item(REGION, AttributeValue::S(region.to_string()))
                    .item(CURRENT_LIST, AttributeValue::N(new_count.to_string()))
                    .send()
                    .await?;

                Ok(new_count)
            }
            None => {
                self.client
                    .put_item()
                    .table_name(&self.refresh_tracking)
                    .item(REGION, AttributeValue::S(region.to_string()))
                    .item(CURRENT_LIST, AttributeValue::N("0".to_string()))
                    .send()
                    .await?;
                Ok(0)
            }
        }
    }
}
