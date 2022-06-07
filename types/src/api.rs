use serde_derive::Deserialize;
use serde_derive::Serialize;
use uuid::Uuid;

#[derive(ts_rs::TS)]
#[ts(export, export_to = "../maccas-web/src/types/Offer.ts")]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Offer {
    pub deal_uuid: String,
    pub count: u32,

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
}

impl From<crate::maccas::MaccasOffer> for Offer {
    fn from(offer: crate::maccas::MaccasOffer) -> Self {
        let short_name = offer.name.split('\n').collect::<Vec<&str>>()[0].to_string();

        Self {
            deal_uuid: Uuid::new_v4().to_hyphenated().to_string(),
            count: 1,
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
            image_base_name: offer.image_base_name,
        }
    }
}

#[derive(ts_rs::TS)]
#[ts(export, export_to = "../maccas-web/src/types/RestaurantInformation.ts")]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestaurantInformation {
    pub name: String,
    pub store_number: i64,
}

impl From<crate::maccas::Restaurant> for RestaurantInformation {
    fn from(res: crate::maccas::Restaurant) -> Self {
        Self {
            name: res.name.clone(),
            store_number: res.national_store_number,
        }
    }
}

#[derive(ts_rs::TS)]
#[ts(
    export,
    export_to = "../maccas-web/src/types/LastRefreshInformation.ts"
)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastRefreshInformation {
    pub last_refresh: String,
}
