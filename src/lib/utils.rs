use crate::constants::mc_donalds;
use crypto::{digest::Digest, sha1::Sha1};
use libmaccas::ApiClient;
use uuid::Uuid;

pub async fn remove_all_from_deal_stack_for(
    api_client: &ApiClient<'_>,
    account_name: &String,
) -> Result<(), anyhow::Error> {
    // honestly, we don't want failures here, so we'll probably just suppress them...
    log::info!("{}: trying to clean deal stack", account_name);
    let deal_stack = api_client
        .get_offers_dealstack(mc_donalds::default::OFFSET, &mc_donalds::default::STORE_ID)
        .await;
    if let Ok(deal_stack) = deal_stack {
        if let Some(deal_stack) = deal_stack.body.response {
            if let Some(deal_stack) = deal_stack.deal_stack {
                for deal in deal_stack {
                    log::info!("{}: removing offer -> {}", account_name, deal.offer_id);
                    api_client
                        .remove_from_offers_dealstack(
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
    Ok(())
}

pub fn get_uuid() -> String {
    Uuid::new_v4().as_hyphenated().to_string()
}

pub fn get_short_sha1(key: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.input_str(key);
    hasher.result_str()[..6].to_owned()
}
