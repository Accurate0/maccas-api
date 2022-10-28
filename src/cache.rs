use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use libmaccas::{types::response::OfferDetailsResponse, ApiClient};

lazy_static! {
    static ref OFFER_DETAILS_CACHE: Arc<Mutex<HashMap<i64, OfferDetailsResponse>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

pub async fn get_offer_details(
    api_client: &ApiClient<'_>,
    offer_proposition_id: i64,
) -> Result<OfferDetailsResponse, anyhow::Error> {
    if let Some(v) = OFFER_DETAILS_CACHE
        .lock()
        .unwrap()
        .get(&offer_proposition_id)
    {
        log::info!(
            "[get_offer_details] loading {} from cache",
            offer_proposition_id
        );
        return Ok(v.clone());
    }

    log::info!(
        "[get_offer_details] cache miss for {}",
        offer_proposition_id
    );
    let result = api_client.offer_details(&offer_proposition_id).await?;
    OFFER_DETAILS_CACHE
        .lock()
        .unwrap()
        .insert(offer_proposition_id, result.body.clone());

    Ok(result.body)
}
