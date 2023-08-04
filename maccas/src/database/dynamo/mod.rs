use crate::types::config::{Indexes, Tables};

pub struct DynamoDatabase {
    client: aws_sdk_dynamodb::Client,
    table_name: String,
    user_config_table_name: String,
    cache_table_name: String,
    cache_table_name_v2: String,
    offer_id_table_name: String,
    point_table_name: String,
    refresh_tracking: String,
    audit: String,
    audit_data: String,
    user_accounts: String,

    audit_user_id_index: String,
}

impl DynamoDatabase {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables, indexes: &Indexes) -> Self {
        Self {
            client,
            table_name: tables.token_cache.to_owned(),
            user_config_table_name: tables.user_config.to_owned(),
            cache_table_name: tables.account_cache.to_owned(),
            cache_table_name_v2: tables.deal_cache.to_owned(),
            offer_id_table_name: tables.locked_offers.to_owned(),
            point_table_name: tables.points.to_owned(),
            refresh_tracking: tables.refresh_tracking.to_owned(),
            audit: tables.audit.to_owned(),
            audit_user_id_index: indexes.audit_user_id_index.to_owned(),
            audit_data: tables.last_refresh.to_owned(),
            user_accounts: tables.user_accounts.to_owned(),
        }
    }
}

mod db;
pub use db::*;
