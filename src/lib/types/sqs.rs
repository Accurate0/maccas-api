use serde::{Deserialize, Serialize};

use super::config::UserAccount;

#[derive(Deserialize, Serialize, Debug)]
pub struct CleanupMessage {
    pub deal_uuid: String,
    pub store_id: i64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FixAccountMessage {
    pub account: UserAccount,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SqsMessage {
    pub body: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SqsEvent {
    #[serde(rename = "Records")]
    pub records: Vec<SqsMessage>,
}
