use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deal {
    offer_id: i64,
    local_valid_from: String,
    local_valid_to: String,
    valid_from_utc: String,
    valid_to_utc: String,
    name: String,
    creation_date_utc: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DealResponse {
    deals: Vec<Deal>,
}
