use crate::types::user::UserOptions;
use foundation::util;
use libmaccas::types::response::PointInformationResponse;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use strum_macros::EnumString;
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

        let base_name_with_webp =
            format!("{}.webp", util::remove_extension(&offer.image_base_name));

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

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PointsDatabase {
    pub total_points: i64,
    pub life_time_points: i64,
}

impl From<PointInformationResponse> for PointsDatabase {
    fn from(res: PointInformationResponse) -> Self {
        Self {
            total_points: res.total_points,
            life_time_points: res.life_time_points,
        }
    }
}
#[derive(Serialize, Deserialize, Debug, Hash, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UserAccountDatabase {
    pub account_name: String,
    pub login_username: String,
    pub login_password: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub group: String,
}

impl Display for UserAccountDatabase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.login_username))
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserOptionsDatabase {
    pub store_id: String,
    pub store_name: Option<String>,
}

impl From<UserOptions> for UserOptionsDatabase {
    fn from(u: UserOptions) -> Self {
        Self {
            store_id: u.store_id,
            store_name: u.store_name,
        }
    }
}

impl From<UserOptionsDatabase> for UserOptions {
    fn from(val: UserOptionsDatabase) -> Self {
        UserOptions {
            store_id: val.store_id,
            store_name: val.store_name,
        }
    }
}

#[derive(Debug, EnumString, Clone, PartialEq, Eq)]
pub enum AuditActionType {
    Add,
    Remove,
}

impl Display for AuditActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
}
