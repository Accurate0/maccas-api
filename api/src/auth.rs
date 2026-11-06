use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("jwks error: {0}")]
    Jwks(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provider {
    pub issuer: String,
    pub audience: String,
}

impl std::str::FromStr for Provider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (issuer, audience) = s
            .rsplit_once('=')
            .filter(|(issuer, audience)| !issuer.is_empty() && !audience.is_empty())
            .ok_or_else(|| format!("expected <issuer>=<audience>, got {s:?}"))?;
        Ok(Self {
            issuer: issuer.to_owned(),
            audience: audience.to_owned(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub providers: Vec<Provider>,
    pub groups: Vec<String>,
}

#[derive(Deserialize)]
struct Issuer {
    iss: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub iat: i64,
    pub exp: i64,
    #[serde(default)]
    pub preferred_username: Option<String>,
    #[serde(default)]
    pub groups: Option<Vec<String>>,
}

impl Claims {
    fn identity(&self) -> &str {
        self.preferred_username.as_deref().unwrap_or(&self.sub)
    }
}

#[derive(Deserialize)]
struct UserInfo {
    #[serde(default)]
    groups: Vec<String>,
}

// kanidm reports groups as SPNs (name@domain) or UUIDs
fn groups_match(held: &[String], allowed: &[String]) -> bool {
    held.iter().any(|group| {
        let name = group
            .split_once('@')
            .map_or(group.as_str(), |(name, _)| name);
        allowed.iter().any(|a| a == group || a == name)
    })
}

#[derive(Clone)]
struct VerifyingKey {
    key: DecodingKey,
    alg: Option<Algorithm>,
}

#[derive(Clone)]
pub struct Authenticator {
    inner: Arc<Inner>,
}

struct Inner {
    client: reqwest::Client,
    groups: Vec<String>,
    issuers: Vec<IssuerKeys>,
}

struct IssuerKeys {
    provider: Provider,
    jwks_uri: String,
    userinfo_endpoint: String,
    keys: RwLock<BTreeMap<String, VerifyingKey>>,
}

impl Authenticator {
    const REFRESH_INTERVAL: Duration = Duration::from_secs(3600);

    pub fn new(config: AuthConfig) -> Self {
        let issuers = config
            .providers
            .into_iter()
            .map(|provider| {
                let base = provider.issuer.trim_end_matches('/').to_owned();
                IssuerKeys {
                    jwks_uri: format!("{base}/public_key.jwk"),
                    userinfo_endpoint: format!("{base}/userinfo"),
                    provider,
                    keys: RwLock::new(BTreeMap::new()),
                }
            })
            .collect();
        Self {
            inner: Arc::new(Inner {
                client: reqwest::Client::builder()
                    .timeout(Duration::from_secs(20))
                    .user_agent("maccas-api")
                    .build()
                    .unwrap_or_default(),
                groups: config.groups,
                issuers,
            }),
        }
    }

    fn issuer_for(&self, token: &str) -> Option<&IssuerKeys> {
        let Issuer { iss } = jsonwebtoken::dangerous::insecure_decode_claims(token).ok()?;
        self.inner.issuers.iter().find(|i| i.provider.issuer == iss)
    }

    pub fn issued(&self, token: &str) -> bool {
        self.issuer_for(token).is_some()
    }

    pub async fn authorize(&self, token: &str) -> Result<Claims, AuthError> {
        let issuer = self
            .issuer_for(token)
            .ok_or_else(|| AuthError::Unauthorized("untrusted issuer".to_owned()))?;
        let claims = self.verify(issuer, token).await?;

        let held = match &claims.groups {
            Some(groups) => groups.clone(),
            None => self.userinfo_groups(issuer, token).await?,
        };
        let allowed = &self.inner.groups;
        if !groups_match(&held, allowed) {
            return Err(AuthError::Forbidden(format!(
                "{} is not in any of {allowed:?}",
                claims.identity()
            )));
        }
        Ok(claims)
    }

    async fn userinfo_groups(
        &self,
        issuer: &IssuerKeys,
        token: &str,
    ) -> Result<Vec<String>, AuthError> {
        let info: UserInfo = self
            .inner
            .client
            .get(&issuer.userinfo_endpoint)
            .bearer_auth(token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(info.groups)
    }

    pub async fn refresh(&self) -> Result<usize, AuthError> {
        let mut total = 0;
        let mut failure = None;
        for issuer in &self.inner.issuers {
            match self.refresh_issuer(issuer).await {
                Ok(count) => total += count,
                Err(e) => {
                    tracing::warn!(issuer = issuer.provider.issuer, "jwks refresh failed: {e}");
                    failure = Some(e);
                }
            }
        }
        match failure {
            Some(e) if total == 0 => Err(e),
            _ => Ok(total),
        }
    }

    async fn refresh_issuer(&self, issuer: &IssuerKeys) -> Result<usize, AuthError> {
        let set: JwkSet = self
            .inner
            .client
            .get(&issuer.jwks_uri)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut keys = BTreeMap::new();
        for jwk in &set.keys {
            let Some(kid) = jwk.common.key_id.clone() else {
                continue;
            };
            let asymmetric = matches!(
                jwk.algorithm,
                AlgorithmParameters::RSA(_)
                    | AlgorithmParameters::EllipticCurve(_)
                    | AlgorithmParameters::OctetKeyPair(_)
            );
            if !asymmetric {
                tracing::warn!("ignoring non-asymmetric jwk {kid}");
                continue;
            }
            match DecodingKey::from_jwk(jwk) {
                Ok(key) => {
                    let alg = jwk
                        .common
                        .key_algorithm
                        .and_then(|a| a.to_string().parse::<Algorithm>().ok());
                    keys.insert(kid, VerifyingKey { key, alg });
                }
                Err(e) => tracing::warn!("ignoring unusable jwk {kid}: {e}"),
            }
        }

        if keys.is_empty() {
            return Err(AuthError::Jwks(format!(
                "no usable keys at {}",
                issuer.jwks_uri
            )));
        }

        let count = keys.len();
        *issuer.keys.write().await = keys;
        tracing::info!(
            issuer = issuer.provider.issuer,
            keys = count,
            "refreshed jwks"
        );
        Ok(count)
    }

    pub async fn run(self) {
        let mut ticker = tokio::time::interval(Self::REFRESH_INTERVAL);
        ticker.tick().await;
        loop {
            ticker.tick().await;
            if let Err(e) = self.refresh().await {
                tracing::warn!("jwks refresh failed, keeping previous keys: {e}");
            }
        }
    }

    async fn verify(&self, issuer: &IssuerKeys, token: &str) -> Result<Claims, AuthError> {
        let header = decode_header(token)
            .map_err(|e| AuthError::Unauthorized(format!("unreadable token header: {e}")))?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::Unauthorized("token has no kid".to_owned()))?;

        let mut key = issuer.keys.read().await.get(&kid).cloned();
        if key.is_none() {
            tracing::info!("unknown kid {kid}, refreshing jwks");
            self.refresh_issuer(issuer).await?;
            key = issuer.keys.read().await.get(&kid).cloned();
        }
        let key = key.ok_or_else(|| AuthError::Unauthorized(format!("no key for kid {kid}")))?;

        let mut validation = Validation::new(key.alg.unwrap_or(header.alg));
        validation.set_issuer(&[&issuer.provider.issuer]);
        validation.set_audience(&[&issuer.provider.audience]);

        decode::<Claims>(token, &key.key, &validation)
            .map(|data| data.claims)
            .map_err(|e| AuthError::Unauthorized(format!("token rejected: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_match_by_short_name_or_spn() {
        let held = vec![
            "00000000-0000-0000-0000-000000000002".to_owned(),
            "platform_admins@idm.example.test".to_owned(),
        ];
        assert!(groups_match(&held, &["platform_admins".to_owned()]));
        assert!(groups_match(
            &held,
            &["platform_admins@idm.example.test".to_owned()]
        ));
        assert!(!groups_match(&held, &["maccas_users".to_owned()]));
        assert!(!groups_match(&[], &["platform_admins".to_owned()]));
    }

    #[test]
    fn providers_parse_issuer_and_audience() {
        assert_eq!(
            "https://idm.example.test/oauth2/openid/graph=graph"
                .parse::<Provider>()
                .unwrap(),
            Provider {
                issuer: "https://idm.example.test/oauth2/openid/graph".to_owned(),
                audience: "graph".to_owned(),
            }
        );
        assert!("https://idm.example.test".parse::<Provider>().is_err());
        assert!("https://idm.example.test=".parse::<Provider>().is_err());
    }

    #[test]
    fn hmac_tokens_are_not_issued_by_kanidm() {
        let auth = Authenticator::new(AuthConfig {
            providers: vec![
                "https://idm.example.test/oauth2/openid/graph=graph"
                    .parse()
                    .unwrap(),
            ],
            groups: vec!["platform_admins".to_owned()],
        });
        assert!(!auth.issued("not-a-jwt"));
    }
}
