// literally a bunch of types, we won't always use everything so..
#![allow(dead_code)]

use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct Token {
    pub token: String,
    pub expires: u32,
}

#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct TokenResponse {
    pub status: Status,
    pub response: Token,
}

#[derive(serde::Deserialize, std::fmt::Debug)]
#[serde(rename_all = "camelCase")]
pub struct AccessTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(serde::Deserialize, std::fmt::Debug)]
pub struct LoginResponse {
    pub status: Status,
    pub response: AccessTokenResponse,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferResponse {
    pub status: Status,
    pub response: Option<OfferList>,
}

#[derive(ts_rs::TS, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    #[serde(skip_serializing, skip_deserializing)]
    pub code: i64,
    #[serde(rename = "type", skip_serializing, skip_deserializing)]
    pub type_field: String,
    #[serde(rename = "correlationID", skip_serializing, skip_deserializing)]
    pub correlation_id: Option<String>,
    pub message: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferList {
    pub offers: Vec<MaccasOffer>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MaccasOffer {
    pub offer_id: i64,
    pub offer_proposition_id: i64,
    #[serde(skip_serializing, skip_deserializing)]
    pub offer_type: i64,
    pub local_valid_from: String,
    pub local_valid_to: String,
    #[serde(rename = "validFromUTC")]
    pub valid_from_utc: String,
    #[serde(rename = "validToUTC")]
    pub valid_to_utc: String,
    pub name: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub short_description: String,
    pub long_description: String,
    pub image_base_name: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub image_base_language: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub redemption_mode: i64,
    #[serde(skip_serializing, skip_deserializing)]
    pub is_archived: bool,
    #[serde(rename = "isSLPOffer", skip_serializing, skip_deserializing)]
    pub is_slpoffer: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub is_locked: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub is_redeemed: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub offer_bucket: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub punch_info: PunchInfo,
    #[serde(skip_serializing, skip_deserializing)]
    pub recurring_info: RecurringInfo,
    #[serde(skip_serializing, skip_deserializing)]
    pub conditions: Conditions,
    #[serde(skip_serializing, skip_deserializing)]
    pub color_coding_info: i64,
    #[serde(skip_serializing, skip_deserializing)]
    pub isvalid_total_order: bool,
    #[serde(rename = "CreationDateUtc")]
    pub creation_date_utc: String,
    #[serde(rename = "extendToEOD", skip_serializing, skip_deserializing)]
    pub extend_to_eod: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub is_dynamic_expiration: bool,
    #[serde(skip_serializing, skip_deserializing)]
    pub daypart_filters: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PunchInfo {
    pub total_punch: i64,
    pub current_punch: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecurringInfo {
    pub total_redemption_quantity: Option<i64>,
    pub current_day_redemption_quantity: Option<i64>,
    pub current_week_redemption_quantity: Option<i64>,
    pub current_month_redemption_quantity: Option<i64>,
    pub max_redemption_quantity: Option<i64>,
    pub max_redemption_quantity_per_day: Option<i64>,
    pub max_redemption_quantity_per_week: Option<i64>,
    pub max_redemption_quantity_per_month: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conditions {
    pub day_of_week_conditions: Vec<String>,
    pub date_conditions: Vec<Value>,
    pub sale_amount_conditions: Vec<SaleAmountCondition>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleAmountCondition {
    pub include_eligible: bool,
    pub minimum: i64,
    pub pre_tax_validation: bool,
    pub include_non_product: bool,
    pub exclude_codes: String,
    pub include_gift_coupons: bool,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestaurantLocationResponse {
    pub status: Status,
    pub response: Option<RestaurantLocationList>,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestaurantLocationList {
    pub restaurants: Vec<Restaurant>,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Restaurant {
    pub restaurant_status: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub facilities: Vec<String>,
    pub address: Address,
    #[serde(skip_serializing, skip_deserializing)]
    pub mc_deliveries: McDeliveries,
    pub location: Location,
    pub name: String,
    pub national_store_number: i64,
    pub status: i64,
    pub time_zone: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub week_opening_hours: Vec<WeekOpeningHour>,
    pub phone_number: Option<String>,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub address_line1: String,
    pub city_town: String,
    pub country: String,
    pub postal_zip: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McDeliveries {
    pub mc_delivery: Vec<Value>,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeekOpeningHour {
    pub services: Vec<Service>,
    pub day_of_week_id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub end_time: String,
    pub is_open: bool,
    pub service_name: String,
    pub start_time: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferDetailsResponse {
    pub status: Status,
    pub response: Option<OfferDetails>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferDetails {
    pub order_discount_type: i64,
    pub offer_proposition_id: i64,
    pub offer_type: i64,
    pub offer_bucket: String,
    pub is_locked: bool,
    pub isvalid_total_order: bool,
    #[serde(rename = "isSLPOffer")]
    pub is_slpoffer: bool,
    pub color_coding_info: i64,
    pub local_valid_from: String,
    pub local_valid_to: String,
    #[serde(rename = "validFromUTC")]
    pub valid_from_utc: String,
    #[serde(rename = "validToUTC")]
    pub valid_to_utc: String,
    pub name: String,
    pub short_description: String,
    pub long_description: String,
    pub image_base_name: String,
    pub image_base_language: String,
    pub redemption_mode: i64,
    pub is_expired: bool,
    pub product_sets: Vec<ProductSet>,
    pub restaurants: Vec<Value>,
    pub frequency_offer_info: FrequencyOfferInfo,
    pub recurring_info: RecurringInfo,
    pub conditions: Conditions,
    pub is_dynamic_expiration: bool,
    #[serde(rename = "exclusiveTOD")]
    pub exclusive_tod: bool,
    pub daypart_filters: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductSet {
    pub alias: String,
    pub quantity: i64,
    pub min_quantity: i64,
    pub products: Vec<String>,
    pub action: Action,
    pub swap_mapping: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(rename = "type")]
    pub type_field: i64,
    pub discount_type: i64,
    pub value: f64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FrequencyOfferInfo {
    pub total_punch: i64,
}

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferDealStackResponse {
    pub status: Status,
    pub response: Option<OfferDealStack>,
}

#[derive(ts_rs::TS, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct OfferDealStack {
    pub random_code: String,
    #[serde(skip_serializing, skip_deserializing)]
    pub bar_code_content: String,
    pub expiration_time: String,
    pub deal_stack: Option<Vec<DealStack>>,
}

#[derive(ts_rs::TS, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct DealStack {
    pub offer_id: i64,
    pub offer_proposition_id: String,
    pub state: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRefreshResponse {
    pub response: Option<LoginRefreshResp>,
    pub status: Status,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRefreshResp {
    pub access_token: String,
    pub refresh_token: String,
}
