use crate::database::types::OfferDatabase;

pub struct RefreshOfferCache {
    pub failed_accounts: Vec<String>,
    pub new_offers: Vec<OfferDatabase>,
}
