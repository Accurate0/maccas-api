use serde_derive::Deserialize;
use serde_derive::Serialize;
use uuid::Uuid;

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Hash)]
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

impl From<libmaccas::types::Offer> for Offer {
    fn from(offer: libmaccas::types::Offer) -> Self {
        let short_name = offer
            .name
            .split('\n')
            .collect::<Vec<&str>>()
            .first()
            .unwrap_or(&offer.name.as_str())
            .to_string();

        Self {
            deal_uuid: Uuid::new_v4().as_hyphenated().to_string(),
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
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestaurantAddress {
    pub address_line: String,
}

impl From<libmaccas::types::Address> for RestaurantAddress {
    fn from(res: libmaccas::types::Address) -> Self {
        Self {
            address_line: res.address_line1,
        }
    }
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestaurantInformation {
    pub name: String,
    pub store_number: i64,
    pub address: RestaurantAddress,
}

impl From<libmaccas::types::Restaurant> for RestaurantInformation {
    fn from(res: libmaccas::types::Restaurant) -> Self {
        Self {
            name: res.name.clone(),
            store_number: res.national_store_number,
            address: RestaurantAddress::from(res.address),
        }
    }
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastRefreshInformation {
    pub last_refresh: String,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub message: String,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferResponse {
    pub random_code: String,
    pub message: String,
}

impl From<libmaccas::types::OfferDealStackResponse> for OfferResponse {
    fn from(res: libmaccas::types::OfferDealStackResponse) -> Self {
        Self {
            random_code: res.response.expect("must have deal stack response").random_code,
            message: res.status.message,
        }
    }
}
