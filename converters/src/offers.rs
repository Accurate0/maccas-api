use crate::{utils, ConversionError, Database};
use entity::offer_details::Model as OfferDetails;
use entity::offers::Model as Offers;
use libmaccas::types::response::OfferDetails as OfferDetailsResponse;
use sea_orm::prelude::{DateTime, Uuid};

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
            if let Some(action) = &item.action {
                action.value + accumulator
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

        let base_name_with_webp =
            format!("{}.webp", utils::remove_extension(&offer.image_base_name));

        let now = chrono::offset::Utc::now().naive_utc();

        Ok(Database(OfferDetails {
            proposition_id: offer.offer_proposition_id,
            name: offer.name.clone(),
            short_name,
            description: offer.long_description.clone(),
            price: if total_price == 0.0 {
                None
            } else {
                Some(total_price)
            },
            image_base_name: base_name_with_webp,
            original_image_base_name: offer.image_base_name.clone(),
            created_at: now,
            updated_at: now,
        }))
    }
}
