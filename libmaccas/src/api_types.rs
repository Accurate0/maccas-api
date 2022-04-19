use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deal {
    pub offer_id: i64,
    pub local_valid_from: String,
    pub local_valid_to: String,
    pub valid_from_utc: String,
    pub valid_to_utc: String,
    pub name: String,
    pub creation_date_utc: String,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DealResponse {
    pub deals: Vec<Deal>,
}
