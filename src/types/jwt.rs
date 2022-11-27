use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Privileged,
    #[default]
    None,
}

impl UserRole {
    pub fn is_allowed_protected_access(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Privileged)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}

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
    #[serde(rename = "extension_Role", default)]
    pub extension_role: UserRole,
}
