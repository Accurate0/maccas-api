use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::constants::mc_donalds;
use lazy_static::lazy_static;
use libmaccas::{types::response::OfferDetailsResponse, ApiClient};

#[async_trait]
pub trait ApiClientExtensions {
    async fn remove_all_from_deal_stack(&self);
    async fn get_offer_details(
        &self,
        offer_proposition_id: i64,
    ) -> Result<OfferDetailsResponse, anyhow::Error>;
}

lazy_static! {
    static ref OFFER_DETAILS_CACHE: Arc<Mutex<HashMap<i64, OfferDetailsResponse>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[async_trait]
impl ApiClientExtensions for ApiClient {
    async fn remove_all_from_deal_stack(&self) {
        // honestly, we don't want failures here, so we'll probably just suppress them...
        let deal_stack = self
            .get_offers_dealstack(mc_donalds::default::OFFSET, &mc_donalds::default::STORE_ID)
            .await;
        if let Ok(deal_stack) = deal_stack {
            if let Some(deal_stack) = deal_stack.body.response {
                if let Some(deal_stack) = deal_stack.deal_stack {
                    for deal in deal_stack {
                        self.remove_from_offers_dealstack(
                            &deal.offer_id,
                            &deal.offer_proposition_id,
                            mc_donalds::default::OFFSET,
                            &mc_donalds::default::STORE_ID,
                        )
                        .await
                        .ok();
                    }
                }
            }
        };
    }

    async fn get_offer_details(
        &self,
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
        let result = self.offer_details(&offer_proposition_id).await?;
        OFFER_DETAILS_CACHE
            .lock()
            .unwrap()
            .insert(offer_proposition_id, result.body.clone());

        Ok(result.body)
    }
}
