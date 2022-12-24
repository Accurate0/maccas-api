use crate::constants::IMAGE_CDN;
use crate::database::types::OfferDatabase;
use crate::database::types::PointsDatabase;
use crate::utils::get_short_sha1;
use itertools::Itertools;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[derive(ToSchema)]
pub struct GetDealsOffer {
    pub deal_uuid: String,
    pub count: u32,
    pub valid_from_utc: String,
    pub valid_to_utc: String,
    pub name: String,
    pub short_name: String,
    pub description: String,
    pub creation_date_utc: String,
    pub price: Option<f64>,
    pub image_url: String,
}

impl From<OfferDatabase> for GetDealsOffer {
    fn from(offer: OfferDatabase) -> Self {
        let image_url = format!("{}/{}", IMAGE_CDN, offer.image_base_name);

        Self {
            deal_uuid: offer.deal_uuid,
            count: 1,
            valid_from_utc: offer.valid_from_utc,
            valid_to_utc: offer.valid_to_utc,
            name: offer.name,
            short_name: offer.short_name,
            description: offer.description,
            creation_date_utc: offer.creation_date_utc,
            price: offer.price,
            image_url,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(ToSchema)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(ToSchema)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(ToSchema)]
pub struct LastRefreshInformation {
    pub last_refresh: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(ToSchema)]
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(ToSchema)]
pub struct OfferPointsResponse {
    pub offer_response: OfferResponse,
    pub points_response: PointsResponse,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AccountResponse(HashMap<String, i64>);

impl From<HashMap<String, Vec<OfferDatabase>>> for AccountResponse {
    fn from(res: HashMap<String, Vec<OfferDatabase>>) -> Self {
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct TotalAccountsResponse(pub i64);

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PointsResponse {
    pub total_points: i64,
    pub life_time_points: i64,
}

impl From<PointsDatabase> for PointsResponse {
    fn from(res: PointsDatabase) -> Self {
        Self {
            total_points: res.total_points,
            life_time_points: res.life_time_points,
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountPointMap {
    pub name: String,
    pub total_points: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AccountPointResponse(Vec<AccountPointMap>);

impl From<HashMap<String, PointsDatabase>> for AccountPointResponse {
    fn from(res: HashMap<String, PointsDatabase>) -> Self {
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

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct AdminLockedDealsResponse(pub Vec<String>);

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct UserSpending {
    pub total: f64,
    pub items: Vec<GetDealsOffer>,
}
