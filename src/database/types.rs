use crate::utils;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(ToSchema)]
pub struct OfferDatabase {
    pub deal_uuid: String,

    pub offer_id: i64,
    pub offer_proposition_id: i64,
    pub local_valid_from: String,
    pub local_valid_to: String,
    #[serde(rename = "validFromUTC")]
    pub valid_from_utc: String,
    #[serde(rename = "validToUTC")]
    pub valid_to_utc: String,
    pub name: String,
    pub short_name: String,
    pub description: String,
    #[serde(rename = "CreationDateUtc")]
    pub creation_date_utc: String,
    pub image_base_name: String,
    pub original_image_base_name: String,

    pub price: Option<f64>,
}

impl From<libmaccas::types::response::Offer> for OfferDatabase {
    fn from(offer: libmaccas::types::response::Offer) -> Self {
        let short_name = offer
            .name
            .split('\n')
            .collect::<Vec<&str>>()
            .first()
            .unwrap_or(&offer.name.as_str())
            .to_string();

        let base_name_with_webp = format!("{}.webp", utils::remove_ext(&offer.image_base_name));

        Self {
            deal_uuid: Uuid::new_v4().as_hyphenated().to_string(),
            offer_id: offer.offer_id,
            offer_proposition_id: offer.offer_proposition_id,
            local_valid_from: offer.local_valid_from,
            local_valid_to: offer.local_valid_to,
            valid_from_utc: offer.valid_from_utc,
            valid_to_utc: offer.valid_to_utc,
            name: offer.name,
            short_name,
            description: offer.long_description,
            creation_date_utc: offer.creation_date_utc,
            image_base_name: base_name_with_webp,
            original_image_base_name: offer.image_base_name,
            price: None,
        }
    }
}

impl PartialEq for OfferDatabase {
    fn eq(&self, other: &Self) -> bool {
        // Everything except for count, offer_id, or uuid, creation_date for equality checks
        self.offer_proposition_id == other.offer_proposition_id
            && self.local_valid_from == other.local_valid_from
            && self.local_valid_to == other.local_valid_to
            && self.valid_from_utc == other.valid_from_utc
            && self.valid_to_utc == other.valid_to_utc
            && self.name == other.name
            && self.short_name == other.short_name
            && self.description == other.description
            && self.image_base_name == other.image_base_name
    }
}
