use foundation::types::role::UserRole;
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JwtClaim {
    pub exp: i64,
    pub iss: String,
    // user id
    pub sub: String,
    // application id
    pub aud: String,
    // issued at
    pub iat: i64,
    // userid
    pub oid: String,
    // bonus
    pub role: UserRole,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SharedTokenClaims {
    pub iss: String,
    pub aud: String,
    pub iat: i64,
    pub role: UserRole,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaAuthorizerPayload {
    pub headers: Headers,
    pub raw_path: String,
    pub raw_query_string: String,
    pub request_context: RequestContext,
    pub route_arn: String,
    pub route_key: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub version: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestContext {
    pub account_id: String,
    pub api_id: String,
    pub domain_name: String,
    pub domain_prefix: String,
    pub http: Http,
    pub request_id: String,
    pub route_key: String,
    pub stage: String,
    pub time: String,
    pub time_epoch: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Headers {
    #[serde(default)]
    pub authorization: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Http {
    pub method: String,
    pub path: String,
    pub protocol: String,
    pub source_ip: String,
    pub user_agent: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LambdaAuthorizerResponse {
    pub is_authorized: bool,
}
