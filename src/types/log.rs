use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageLog<'a> {
    pub user_id: String,
    pub deal_readable: String,
    pub deal_uuid: String,
    pub user_readable: String,
    pub message: &'a str,
    pub local_time: String,
}