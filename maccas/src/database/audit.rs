use super::types::{AuditActionType, OfferDatabase};
use crate::{
    constants::db::{
        ACTION, ACTOR, DEAL_UUID, OFFER, OFFER_NAME, OPERATION_ID, TIMESTAMP, USER_ID, USER_NAME,
    },
    types::{
        audit::AuditEntry,
        config::{Indexes, Tables},
    },
};
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use std::{str::FromStr, time::SystemTime};

#[derive(Clone)]
struct Index {
    audit_user_id: String,
}

#[derive(Clone)]
pub struct AuditRepository {
    client: aws_sdk_dynamodb::Client,
    audit: String,
    index: Index,
}

impl AuditRepository {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables, indexes: &Indexes) -> Self {
        Self {
            client,
            audit: tables.audit.clone(),
            index: Index {
                audit_user_id: indexes.audit_user_id_index.clone(),
            },
        }
    }

    pub async fn add_entry(
        &self,
        action: AuditActionType,
        user_id: Option<String>,
        actor: String,
        offer: &OfferDatabase,
    ) {
        let user_id = user_id.unwrap_or_else(|| "unknown".to_owned());

        log::info!("adding to audit table: {user_id}/{actor} {:?}", offer);

        let now = SystemTime::now();
        let now: DateTime<Utc> = now.into();
        let now = now.to_rfc3339();
        let offer_attribute = serde_dynamo::to_item(offer);

        if let Err(e) = offer_attribute {
            log::error!("error adding to audit table: {e}");
            return;
        }

        if let Err(e) = self
            .client
            .put_item()
            .table_name(&self.audit)
            .item(
                OPERATION_ID,
                AttributeValue::S(uuid::Uuid::now_v7().to_string()),
            )
            .item(ACTION, AttributeValue::S(action.to_string()))
            .item(DEAL_UUID, AttributeValue::S(offer.deal_uuid.to_string()))
            .item(USER_ID, AttributeValue::S(user_id))
            .item(USER_NAME, AttributeValue::S(actor.clone()))
            .item(ACTOR, AttributeValue::S(actor))
            .item(OFFER_NAME, AttributeValue::S(offer.short_name.to_string()))
            .item(TIMESTAMP, AttributeValue::S(now))
            .item(OFFER, AttributeValue::M(offer_attribute.unwrap()))
            .send()
            .await
        {
            log::error!("error adding to audit table: {e}")
        };
    }

    pub async fn get_entries(&self, user_id: String) -> Result<Vec<AuditEntry>, anyhow::Error> {
        let resp = self
            .client
            .query()
            .table_name(&self.audit)
            .index_name(&self.index.audit_user_id)
            .key_condition_expression("#user = :user_id")
            .expression_attribute_names("#user", USER_ID)
            .expression_attribute_values(":user_id", AttributeValue::S(user_id.to_string()))
            .send()
            .await?;

        Ok(resp
            .items()
            .iter()
            .map(|item| {
                let action = AuditActionType::from_str(item[ACTION].as_s().unwrap()).unwrap();
                let offer = serde_dynamo::from_item(item[OFFER].as_m().unwrap().clone()).unwrap();
                AuditEntry {
                    action,
                    offer,
                    user_id: user_id.to_string(),
                }
            })
            .collect_vec())
    }
}
