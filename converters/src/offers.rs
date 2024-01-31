use crate::{ConversionError, Database};
use entity::offer_details::Model as OfferDetails;
use entity::offer_history::Model as OfferHistory;
use entity::offers::Model as Offers;
use libmaccas::types::response::OfferDetails as OfferDetailsResponse;
use sea_orm::prelude::{DateTime, Uuid};

impl From<Database<Offers>> for Database<OfferHistory> {
    fn from(offer: Database<Offers>) -> Self {
        let offer = offer.0;

        Database(OfferHistory {
            id: Uuid::new_v4(),
            offer_id: offer.offer_id,
            offer_proposition_id: offer.offer_proposition_id,
            valid_to: offer.valid_to,
            valid_from: offer.valid_from,
            creation_date: offer.creation_date,
            account_id: offer.account_id,
            created_at: offer.created_at,
            updated_at: offer.updated_at,
        })
    }
}

impl Database<Offers> {
    pub fn convert_offer(
        offer: &libmaccas::types::response::Offer,
        account_id: Uuid,
    ) -> Result<Self, ConversionError> {
        let datetime_format = "%FT%TZ";
        let now = chrono::offset::Utc::now().naive_utc();

        Ok(Database(Offers {
            id: Uuid::new_v4(),
            offer_id: offer.offer_id,
            offer_proposition_id: offer.offer_proposition_id,
            valid_to: DateTime::parse_from_str(&offer.valid_to_utc, datetime_format)?,
            valid_from: DateTime::parse_from_str(&offer.valid_from_utc, datetime_format)?,
            creation_date: DateTime::parse_from_str(&offer.creation_date_utc, datetime_format)?,
            account_id,
            created_at: now,
            updated_at: now,
        }))
    }
}

impl Database<OfferDetails> {
    pub fn convert_offer_details(offer: &OfferDetailsResponse) -> Result<Self, ConversionError> {
        let total_price = offer.product_sets.iter().fold(0f64, |accumulator, item| {
            // hash browns are 2 for $2, so quantity is 2, and each product is valued at $1
            // product_sets: [
            //     ProductSet {
            //         alias: Some(
            //             "Item to discount",
            //         ),
            //         quantity: 2,
            //         min_quantity: Some(
            //             2,
            //         ),
            //         products: [
            //             "202",
            //         ],
            //         action: Some(
            //             Action {
            //                 type_field: 3,
            //                 discount_type: 2,
            //                 value: 1.0,
            //             },
            //         ),
            //         swap_mapping: [],
            //     },
            // ],

            if let Some(action) = &item.action {
                (action.value * item.quantity as f64) + accumulator
            } else {
                accumulator
            }
        });

        let short_name = offer
            .name
            .split('\n')
            .collect::<Vec<&str>>()
            .first()
            .unwrap_or(&offer.name.as_str())
            .to_string();

        let now = chrono::offset::Utc::now().naive_utc();

        let product_ids = offer
            .product_sets
            .iter()
            .flat_map(|ps| ps.products.clone())
            .flat_map(|id| id.parse::<i64>())
            .collect::<Vec<_>>();

        Ok(Database(OfferDetails {
            proposition_id: offer.offer_proposition_id,
            name: offer.name.clone().replace(&short_name, ""),
            short_name,
            description: offer.long_description.clone(),
            price: if total_price == 0.0 {
                None
            } else {
                Some(total_price)
            },
            image_base_name: offer.image_base_name.clone(),
            created_at: now,
            updated_at: now,
            raw_data: Some(serde_json::to_value(offer)?),
            products: Some(product_ids),
        }))
    }
}
