use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(ts_rs::TS)]
#[ts(export, export_to = "../maccas-web/src/types/JwtClaim.ts")]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    #[serde(rename = "auth_time")]
    pub auth_time: i64,
    pub oid: String,
    pub name: String,
    pub tfp: String,
}
