use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtClaim {
    pub exp: i64,
    pub nbf: i64,
    pub ver: String,
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub nonce: String,
    pub iat: i64,
    pub oid: String,
    pub name: String,
    pub tfp: String,
    #[serde(rename = "extension_AdminUser")]
    pub extension_admin_user: Option<bool>,
}
