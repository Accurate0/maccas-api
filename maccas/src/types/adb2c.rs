use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Adb2cTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: String,
    pub id_token: String,
}
