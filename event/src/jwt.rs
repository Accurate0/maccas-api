use crate::error::MiddlewareError;
use crate::state::AppState;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::AUTHORIZATION;
use actix_web::{web, Error};
use actix_web_lab::middleware::Next;
use base::jwt::{self, Role};

pub async fn validator(
    state: web::Data<AppState>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .map(|v| v.to_str().map(|t| t.replace("Bearer ", "")));

    match token {
        Some(token) => match jwt::verify_jwt(
            state.settings.auth_secret.as_bytes(),
            &token.map_err(|_| MiddlewareError::Unauthenticated)?,
        ) {
            Ok(claims) => {
                tracing::info!("verified token with claims: {:?}", claims);
                next.call(req).await
            }
            Err(e) => {
                tracing::error!("error validating token: {}", e);
                Err(MiddlewareError::Unauthenticated.into())
            }
        },
        None => {
            if cfg!(debug_assertions) {
                next.call(req).await
            } else {
                Err(MiddlewareError::Unauthenticated.into())
            }
        }
    }
}

pub async fn validator_admin_only(
    state: web::Data<AppState>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .map(|v| v.to_str().map(|t| t.replace("Bearer ", "")));

    match token {
        Some(token) => match jwt::verify_jwt(
            state.settings.auth_secret.as_bytes(),
            &token.map_err(|_| MiddlewareError::Unauthenticated)?,
        ) {
            Ok(claims) if claims.role == Role::Admin => {
                tracing::info!("verified token with claims: {:?}", claims);
                next.call(req).await
            }
            Ok(claims) => {
                tracing::error!("token validated, user is not admin: {:?}", claims);
                Err(MiddlewareError::Unauthenticated.into())
            }
            Err(e) => {
                tracing::error!("error validating token: {}", e);
                Err(MiddlewareError::Unauthenticated.into())
            }
        },
        None => {
            if cfg!(debug_assertions) {
                next.call(req).await
            } else {
                Err(MiddlewareError::Unauthenticated.into())
            }
        }
    }
}
