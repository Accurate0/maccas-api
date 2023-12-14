use crate::{utils, ConversionError, Database};
use entity::offers::Model as Offers;
use sea_orm::prelude::{DateTime, Uuid};

impl Database<Offers> {
    pub fn try_from(
        offer: libmaccas::types::response::Offer,
        account_name: String,
    ) -> Result<Self, ConversionError> {
        let short_name = offer
            .name
            .split('\n')
            .collect::<Vec<&str>>()
            .first()
            .unwrap_or(&offer.name.as_str())
            .to_string();

        let base_name_with_webp =
            format!("{}.webp", utils::remove_extension(&offer.image_base_name));

        let datetime_format = "%FT%TZ";

        Ok(Database(Offers {
            id: Uuid::new_v4(),
            offer_id: offer.offer_id,
            offer_proposition_id: offer.offer_proposition_id,
            name: offer.name,
            short_name,
            description: offer.long_description,
            valid_to: DateTime::parse_from_str(&offer.valid_to_utc, datetime_format)?,
            valid_from: DateTime::parse_from_str(&offer.valid_from_utc, datetime_format)?,
            creation_date: DateTime::parse_from_str(&offer.creation_date_utc, datetime_format)?,
            image_base_name: base_name_with_webp,
            original_image_base_name: offer.image_base_name,
            account_name,
        }))
    }
}
