use crate::types::{ApiState, AppError};
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use base::jwt::{self, JwtClaims, Role};

#[derive(Clone)]
#[allow(dead_code)]
pub struct ValidatedClaims(pub JwtClaims);

pub async fn validate(
    headers: HeaderMap,
    State(state): State<ApiState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = headers.get("Authorization");
    let settings = state.settings;

    if cfg!(debug_assertions) {
        if let Some(auth_header) = auth_header {
            let token = &auth_header.to_str()?.replace("Bearer ", "");

            let validated_claims = jwt::verify_jwt(settings.auth_secret.as_bytes(), token)?;

            request
                .extensions_mut()
                .insert(ValidatedClaims(validated_claims.clone()));
            tracing::info!("verified token with claims: {:?}", validated_claims);

            Some(token.clone())
        } else {
            None
        };

        return Ok(next.run(request).await);
    }

    match auth_header {
        Some(auth_header) => {
            let token = &auth_header.to_str()?.replace("Bearer ", "");

            let claims = jwt::verify_jwt(settings.auth_secret.as_bytes(), token)?;

            request
                .extensions_mut()
                .insert(ValidatedClaims(claims.clone()));
            if claims.role.contains(&Role::Admin) {
                tracing::info!("verified token with claims: {:?}", claims);
                Ok(next.run(request).await)
            } else {
                Err(AppError::StatusCode(StatusCode::UNAUTHORIZED))
            }
        }
        None => Err(AppError::StatusCode(StatusCode::UNAUTHORIZED)),
    }
}
