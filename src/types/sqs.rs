use super::{config::UserAccount, images::OfferImageBaseName};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct CleanupMessage {
    pub deal_uuid: String,
    pub store_id: i64,
    pub user_id: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ImagesRefreshMessage {
    pub image_base_names: Vec<OfferImageBaseName>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RefreshFailureMessage(pub UserAccount);

#[derive(Deserialize, Serialize, Debug)]
pub struct SqsMessage {
    pub body: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SqsEvent {
    #[serde(rename = "Records")]
    pub records: Vec<SqsMessage>,
}
