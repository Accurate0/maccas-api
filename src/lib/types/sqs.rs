use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiMessage {
    pub deal_uuid: String,
    pub store_id: i64,
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
