use crate::utils::get_short_sha1;
use itertools::Itertools;
use libmaccas::types::response::PointInformationResponse;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use utoipa::Component;
use uuid::Uuid;

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Component)]
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

impl From<libmaccas::types::response::Offer> for Offer {
    fn from(offer: libmaccas::types::response::Offer) -> Self {
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

impl PartialEq for Offer {
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

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Component)]
pub struct RestaurantAddress {
    pub address_line: String,
}

impl From<libmaccas::types::response::Address> for RestaurantAddress {
    fn from(res: libmaccas::types::response::Address) -> Self {
        Self {
            address_line: res.address_line1,
        }
    }
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Component)]
pub struct RestaurantInformation {
    pub name: String,
    pub store_number: i64,
    pub address: RestaurantAddress,
}

impl From<libmaccas::types::response::Restaurant> for RestaurantInformation {
    fn from(res: libmaccas::types::response::Restaurant) -> Self {
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
#[derive(Component)]
pub struct LastRefreshInformation {
    pub last_refresh: String,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Component)]
pub struct OfferResponse {
    pub random_code: String,
    pub message: String,
}

impl From<libmaccas::types::response::OfferDealStackResponse> for OfferResponse {
    fn from(res: libmaccas::types::response::OfferDealStackResponse) -> Self {
        Self {
            random_code: res
                .response
                .expect("must have deal stack response")
                .random_code,
            message: res
                .status
                .message
                .unwrap_or_else(|| "No message".to_string()),
        }
    }
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Component)]
pub struct OfferPointsResponse {
    pub offer_response: OfferResponse,
    pub points_response: PointsResponse,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Component)]
pub struct AccountResponse(HashMap<String, i64>);

impl From<HashMap<String, Vec<Offer>>> for AccountResponse {
    fn from(res: HashMap<String, Vec<Offer>>) -> Self {
        let res = res
            .iter()
            .map(|(key, value)| {
                let hash = get_short_sha1(key);
                (hash, value.len() as i64)
            })
            .collect();

        Self(res)
    }
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Component)]
pub struct TotalAccountsResponse(pub i64);

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Component)]
#[serde(rename_all = "camelCase")]
pub struct PointsResponse {
    pub total_points: i64,
    pub life_time_points: i64,
}

impl From<PointInformationResponse> for PointsResponse {
    fn from(res: PointInformationResponse) -> Self {
        Self {
            total_points: res.total_points,
            life_time_points: res.life_time_points,
        }
    }
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Component)]
#[serde(rename_all = "camelCase")]
pub struct AccountPointMap {
    pub name: String,
    pub total_points: i64,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Component)]
pub struct AccountPointResponse(Vec<AccountPointMap>);

impl From<HashMap<String, PointsResponse>> for AccountPointResponse {
    fn from(res: HashMap<String, PointsResponse>) -> Self {
        Self(
            res.iter()
                .map(|(key, value)| AccountPointMap {
                    name: key.to_string(),
                    total_points: value.total_points,
                })
                .sorted_by(|a, b| b.total_points.cmp(&a.total_points))
                .collect(),
        )
    }
}
