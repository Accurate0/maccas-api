use crate::types::config::{Indexes, Tables};

pub struct DynamoDatabase {
    client: aws_sdk_dynamodb::Client,
    token_cache_table_name: String,
    user_config_table_name: String,
    account_cache_table_name: String,
    offer_cache_table_name: String,
    locked_offers_table_name: String,
    point_table_name: String,
    refresh_tracking_table_name: String,
    audit_table_name: String,
    last_refresh_table_name: String,
    user_accounts: String,
    users: String,
    user_tokens: String,
    current_deals: String,

    current_deals_offer_proposition_id: String,
    current_deals_account_name: String,
    audit_user_id_index: String,
}

impl DynamoDatabase {
    pub fn new(client: aws_sdk_dynamodb::Client, tables: &Tables, indexes: &Indexes) -> Self {
        Self {
            client,
            token_cache_table_name: tables.token_cache.to_owned(),
            user_config_table_name: tables.user_config.to_owned(),
            account_cache_table_name: tables.account_cache.to_owned(),
            offer_cache_table_name: tables.deal_cache.to_owned(),
            locked_offers_table_name: tables.locked_offers.to_owned(),
            point_table_name: tables.points.to_owned(),
            refresh_tracking_table_name: tables.refresh_tracking.to_owned(),
            audit_table_name: tables.audit.to_owned(),
            last_refresh_table_name: tables.last_refresh.to_owned(),
            user_accounts: tables.user_accounts.to_owned(),
            users: tables.users.to_owned(),
            user_tokens: tables.user_tokens.to_owned(),
            current_deals: tables.current_deals.to_owned(),

            audit_user_id_index: indexes.audit_user_id_index.to_owned(),
            current_deals_offer_proposition_id: indexes
                .current_deals_offer_proposition_id
                .to_owned(),
            current_deals_account_name: indexes.current_deals_account_name.to_owned(),
        }
    }
}

mod db;
pub use db::*;
