use lambda_http::Error;
use std::collections::HashMap;
use types::maccas::Offer;

pub async fn get_by_order_id<'a>(
    offer_map: &HashMap<&String, Option<Vec<Offer>>>,
    deal_id: &String,
) -> Result<(String, String), Error> {
    let mut offer_account_name: Option<String> = None;
    let mut offer_proposition_id: Option<String> = None;
    for (account_name, offer_list) in offer_map {
        match offer_list {
            None => {}
            Some(offer_list) => {
                for offer in offer_list {
                    if offer.offer_id.to_string() == *deal_id {
                        offer_account_name = Some(account_name.to_string());
                        offer_proposition_id = Some(offer.offer_proposition_id.to_string());
                        break;
                    }
                }
            }
        }
    }

    let offer_account_name = offer_account_name.ok_or("no account")?;
    let offer_proposition_id = offer_proposition_id.ok_or("no offer")?;

    Ok((offer_account_name, offer_proposition_id))
}
