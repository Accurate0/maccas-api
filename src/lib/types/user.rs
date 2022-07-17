use serde_derive::Deserialize;
use serde_derive::Serialize;
use utoipa::Component;

#[derive(ts_rs::TS)]
#[ts(export)]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, Component)]
#[serde(rename_all = "camelCase")]
pub struct UserOptions {
    pub store_id: String,
    pub store_name: Option<String>,
}
