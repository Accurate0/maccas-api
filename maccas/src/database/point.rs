use super::types::{PointsDatabase, UserAccountDatabase};
use crate::{
    constants::db::{ACCOUNT_HASH, ACCOUNT_INFO, ACCOUNT_NAME, LAST_REFRESH, POINT_INFO},
    types::config::Tables,
};
use anyhow::{bail, Context};
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use foundation::hash::get_short_sha1;
use libmaccas::ApiClient;
use std::{collections::HashMap, time::SystemTime};

#[derive(Clone)]
pub struct PointRepository {
    client: aws_sdk_dynamodb::Client,
    points: String,
}

impl PointRepository {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables) -> Self {
        Self {
            client,
            points: tables.points.clone(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn refresh_point_cache(
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
                    .table_name(&self.points)
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

    #[tracing::instrument(skip(self))]
    pub async fn get_all_points(&self) -> Result<HashMap<String, PointsDatabase>, anyhow::Error> {
        let mut point_map = HashMap::<String, PointsDatabase>::new();

        let table_resp: Result<Vec<_>, _> = self
            .client
            .scan()
            .table_name(&self.points)
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

    #[tracing::instrument(skip(self))]
    pub async fn get_points(
        &self,
        account_hash: &str,
    ) -> Result<(UserAccountDatabase, PointsDatabase), anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.points)
            .key_condition_expression("#hash = :account_hash")
            .expression_attribute_names("#hash", ACCOUNT_HASH)
            .expression_attribute_values(
                ":account_hash",
                AttributeValue::S(account_hash.to_string()),
            )
            .send()
            .await?;

        if resp.items().len() == 1 {
            let item = resp.items().first().unwrap();
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
}
