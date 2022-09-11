use super::api::OfferDatabase;

pub struct RefreshOfferCache {
    pub failed_accounts: Vec<String>,
    pub new_offers: Vec<OfferDatabase>,
}
