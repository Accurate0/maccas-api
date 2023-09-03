use super::role::UserRole;
use aliri::jwt::{CoreClaims, IssuerRef, SubjectRef};
use aliri_clock::UnixTime;
use aliri_oauth2::HasScope;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtClaim {
    pub exp: u64,
    pub nbf: u64,
    pub ver: String,
    pub iss: aliri::jwt::Issuer,
    pub sub: aliri::jwt::Subject,
    pub aud: aliri::jwt::Audiences,
    pub nonce: String,
    pub iat: i64,
    pub scp: aliri_oauth2::Scope,
    pub oid: String,
    pub name: String,
    pub tfp: String,
    #[serde(rename = "extension_Role", default)]
    pub extension_role: UserRole,
}

impl HasScope for JwtClaim {
    fn scope(&self) -> &aliri_oauth2::Scope {
        &self.scp
    }
}

impl CoreClaims for JwtClaim {
    fn nbf(&self) -> Option<UnixTime> {
        Some(UnixTime(self.nbf))
    }

    fn exp(&self) -> Option<UnixTime> {
        Some(UnixTime(self.exp))
    }

    fn aud(&self) -> &aliri::jwt::Audiences {
        &self.aud
    }

    fn iss(&self) -> Option<&IssuerRef> {
        Some(&self.iss)
    }

    fn sub(&self) -> Option<&SubjectRef> {
        Some(&self.sub)
    }
}
