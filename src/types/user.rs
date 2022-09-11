use serde_derive::Deserialize;
use serde_derive::Serialize;
use utoipa::ToSchema;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserOptions {
    pub store_id: String,
    pub store_name: Option<String>,
}
