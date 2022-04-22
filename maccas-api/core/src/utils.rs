use lambda_http::Error;
use libmaccas::api::ApiClient;
use std::collections::HashMap;
use types::maccas::Offer;

pub async fn get_by_order_id<'a>(
    offer_map: &HashMap<&String, Vec<Offer>>,
    deal_id: &String,
    client_map: &'a HashMap<String, ApiClient>,
) -> Result<(&'a ApiClient, String, String), Error> {
    let mut offer_account_name: Option<String> = None;
    let mut offer_proposition_id: Option<String> = None;
    for (account_name, offer_list) in offer_map {
        for offer in offer_list {
            if offer.offer_id.to_string() == *deal_id {
                offer_account_name = Some(account_name.to_string());
                offer_proposition_id = Some(offer.offer_proposition_id.to_string());
                break;
            }
        }
    }

    let offer_account_name = offer_account_name.ok_or("no account")?;
    let offer_proposition_id = offer_proposition_id.ok_or("no offer")?;
    let api_client = client_map.get(&offer_account_name).ok_or("no api client")?;

    Ok((api_client, offer_account_name, offer_proposition_id))
}
