use super::types::{
    AuditActionType, OfferDatabase, PointsDatabase, RegistrationTokenMetadata, User,
    UserAccountDatabase, UserOptionsDatabase,
};
use crate::types::{
    audit::AuditEntry, config::GeneralConfig, refresh::RefreshOfferCache, role::UserRole,
};
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
        actor: String,
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
    async fn set_access_and_refresh_token_for(
        &self,
        account_name: &str,
        access_token: &str,
        refresh_token: &str,
    ) -> Result<(), anyhow::Error>;
    async fn is_user_exist(&self, username: String) -> Result<bool, anyhow::Error>;
    async fn create_user(
        &self,
        user_id: String,
        username: String,
        password_hash: String,
        salt: Vec<u8>,
        is_imported: bool,
        registration_token: Option<&str>,
    ) -> Result<(), anyhow::Error>;

    async fn set_user_role(&self, username: String, role: UserRole) -> Result<(), anyhow::Error>;
    async fn set_user_tokens(
        &self,
        username: &str,
        auth_token: &str,
        refresh_token: &str,
        ttl: Duration,
    ) -> Result<(), anyhow::Error>;
    async fn get_password_hash(&self, username: String) -> Result<String, anyhow::Error>;
    async fn get_user_id(&self, username: String) -> Result<String, anyhow::Error>;
    async fn get_user_role(&self, username: String) -> Result<UserRole, anyhow::Error>;
    async fn get_user_tokens(&self, username: String) -> Result<(String, String), anyhow::Error>;
    async fn find_all_by_proposition_id(
        &self,
        proposition_id: &str,
    ) -> Result<Vec<String>, anyhow::Error>;
    async fn get_all_users(&self) -> Result<Vec<User>, anyhow::Error>;
    async fn create_registration_token(
        &self,
        registration_token: &str,
        role: UserRole,
        single_use: bool,
    ) -> Result<(), anyhow::Error>;
    async fn get_registration_token(
        &self,
        registration_token: &str,
    ) -> Result<RegistrationTokenMetadata, anyhow::Error>;
    async fn set_registration_token_use_count(
        &self,
        registration_token: &str,
        count: u32,
    ) -> Result<(), anyhow::Error>;
}
