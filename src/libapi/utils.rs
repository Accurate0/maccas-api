use crate::{constants::mc_donalds, types::api::Offer};
use lambda_http::Error;
use libmaccas::ApiClient;
use std::collections::HashMap;
use uuid::Uuid;

#[deprecated]
pub async fn get_by_order_id<'a>(
    offer_map: &HashMap<String, Vec<Offer>>,
    deal_id: &String,
) -> Result<(String, String, String, String), Error> {
    let mut offer_account_name: Option<String> = None;
    let mut offer_proposition_id: Option<String> = None;
    let mut offer_id: Option<String> = None;
    let mut offer_name: Option<String> = None;

    for (account_name, offer_list) in offer_map {
        for offer in offer_list {
            if *offer.deal_uuid == *deal_id {
                offer_account_name = Some(account_name.to_string());
                offer_proposition_id = Some(offer.offer_proposition_id.to_string());
                offer_id = Some(offer.offer_id.to_string());
                offer_name = Some(offer.name.to_string());
                break;
            }
        }
    }

    let offer_account_name = offer_account_name.ok_or("no account")?;
    let offer_proposition_id = offer_proposition_id.ok_or("no offer")?;
    let offer_id = offer_id.ok_or("no offer id")?;
    let offer_name = offer_name.ok_or("no offer id")?;

    Ok((offer_account_name, offer_proposition_id, offer_id, offer_name))
}

pub async fn remove_all_from_deal_stack_for(api_client: &ApiClient<'_>, account_name: &String) -> Result<(), Error> {
    // honestly, we don't want failures here, so we'll probably just suppress them...
    log::info!("{}: trying to clean deal stack", account_name);
    let deal_stack = api_client
        .get_offers_dealstack(mc_donalds::default::OFFSET, mc_donalds::default::STORE_ID)
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
                            mc_donalds::default::STORE_ID,
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
    Uuid::new_v4().to_hyphenated().to_string()
}
