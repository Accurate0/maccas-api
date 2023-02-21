use super::types::{
    AuditActionType, OfferDatabase, PointsDatabase, UserAccountDatabase, UserOptionsDatabase,
};
use crate::types::{audit::AuditEntry, config::GeneralConfig, refresh::RefreshOfferCache};
use async_trait::async_trait;
use chrono::Duration;
use libmaccas::ApiClient;
use std::collections::HashMap;

#[async_trait]
pub trait Database {
    async fn get_all_offers_as_map(
        &self,
    ) -> Result<HashMap<String, Vec<OfferDatabase>>, anyhow::Error>;
    async fn get_all_offers_as_vec(&self) -> Result<Vec<OfferDatabase>, anyhow::Error>;
    async fn get_offers_for(
        &self,
        account_name: &str,
    ) -> Result<Option<Vec<OfferDatabase>>, anyhow::Error>;
    async fn set_offers_for(
        &self,
        account_name: &UserAccountDatabase,
        offer_list: &[OfferDatabase],
    ) -> Result<(), anyhow::Error>;
    async fn refresh_offer_cache(
        &self,
        client_map: &HashMap<UserAccountDatabase, ApiClient>,
        ignored_offer_ids: &[i64],
    ) -> Result<RefreshOfferCache, anyhow::Error>;
    async fn refresh_point_cache_for(
        &self,
        account: &UserAccountDatabase,
        api_client: &ApiClient,
    ) -> Result<(), anyhow::Error>;
    async fn get_point_map(&self) -> Result<HashMap<String, PointsDatabase>, anyhow::Error>;
    async fn get_points_by_account_hash(
        &self,
        account_hash: &str,
    ) -> Result<(UserAccountDatabase, PointsDatabase), anyhow::Error>;
    async fn refresh_offer_cache_for(
        &self,
        account: &UserAccountDatabase,
        api_client: &ApiClient,
        ignored_offer_ids: &[i64],
    ) -> Result<Vec<OfferDatabase>, anyhow::Error>;
    async fn set_last_refresh(&self) -> Result<(), anyhow::Error>;
    async fn get_last_refresh(&self) -> Result<String, anyhow::Error>;
    async fn get_offer_by_id(
        &self,
        offer_id: &str,
    ) -> Result<(UserAccountDatabase, OfferDatabase), anyhow::Error>;
    async fn get_config_by_user_id(
        &self,
        user_id: &str,
    ) -> Result<UserOptionsDatabase, anyhow::Error>;
    async fn set_config_by_user_id(
        &self,
        user_id: &str,
        user_config: &UserOptionsDatabase,
        user_name: &str,
    ) -> Result<(), anyhow::Error>;
    async fn get_specific_client<'a>(
        &self,
        http_client: reqwest_middleware::ClientWithMiddleware,
        client_id: &'a str,
        client_secret: &'a str,
        sensor_data: &'a str,
        account: &'a UserAccountDatabase,
        force_login: bool,
    ) -> Result<ApiClient, anyhow::Error>;
    async fn get_client_map<'a>(
        &self,
        config: &GeneralConfig,
        client_id: &'a str,
        client_secret: &'a str,
        sensor_data: &'a str,
        account_list: &'a [UserAccountDatabase],
        force_login: bool,
    ) -> Result<(HashMap<UserAccountDatabase, ApiClient>, Vec<String>), anyhow::Error>;
    async fn lock_deal(&self, deal_id: &str, duration: Duration) -> Result<(), anyhow::Error>;
    async fn unlock_deal(&self, deal_id: &str) -> Result<(), anyhow::Error>;
    async fn get_all_locked_deals(&self) -> Result<Vec<String>, anyhow::Error>;
    async fn delete_all_locked_deals(&self) -> Result<(), anyhow::Error>;
    async fn get_device_id_for(&self, account_name: &str) -> Result<Option<String>, anyhow::Error>;
    async fn set_device_id_for(
        &self,
        account_name: &str,
        device_id: &str,
    ) -> Result<(), anyhow::Error>;
    async fn increment_refresh_tracking(
        &self,
        region: &str,
        max_count: i8,
    ) -> Result<i8, anyhow::Error>;
    async fn add_to_audit(
        &self,
        action: AuditActionType,
        user_id: Option<String>,
        user_name: Option<String>,
        offer_id: &OfferDatabase,
    );
    async fn get_audit_entries_for(
        &self,
        user_id: String,
    ) -> Result<Vec<AuditEntry>, anyhow::Error>;
    async fn add_user_account(
        &self,
        account_name: &str,
        login_username: &str,
        login_password: &str,
        region: &str,
        group: &str,
    ) -> Result<(), anyhow::Error>;
    async fn get_account(&self, account_name: &str) -> Result<UserAccountDatabase, anyhow::Error>;
    async fn get_accounts_for_region_and_group(
        &self,
        region: &str,
        group: &str,
    ) -> Result<Vec<UserAccountDatabase>, anyhow::Error>;
}
