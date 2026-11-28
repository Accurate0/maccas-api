use super::tracing::Untraced;
use crate::auth::{AuthError, Authenticator, PASSPORT_HEADER};
use crate::types::{ApiState, AppError};
use anyhow::Context;
use async_graphql::{ServerError, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::StatusCode;
use axum::response::Result;
use axum::{Json, extract::State, http::HeaderMap, response::IntoResponse};
use base::jwt::{self, JwtClaims, Role};

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/v1/graphql").finish()).into_response()
}

#[allow(unused)]
pub struct ValidatedToken(pub String);
pub struct ValidatedClaims(pub JwtClaims);

fn unauthorized(message: impl Into<String>) -> GraphQLResponse {
    let mut response = async_graphql::Response::new(());
    response.errors.push(ServerError::new(message, None));
    GraphQLResponse::from(response)
}

#[tracing::instrument(
    name = "auth.validate_passport",
    skip_all,
    fields(valid = tracing::field::Empty)
)]
async fn passport_claims(auth: &Authenticator, passport: &str) -> Result<JwtClaims, AuthError> {
    let claims = auth.authorize(passport).await;
    tracing::Span::current().record("valid", claims.is_ok());
    let claims = claims?;
    let scopes = claims.scopes();
    let role = if scopes.is_empty() {
        vec![Role::Admin, Role::Points]
    } else {
        [
            ("maccas:offer:write", Role::Admin),
            ("maccas:points:read", Role::Points),
        ]
        .into_iter()
        .filter(|(scope, _)| scopes.contains(scope))
        .map(|(_, role)| role)
        .collect()
    };

    Ok(JwtClaims {
        user_id: String::new(),
        session_id: None,
        iat: claims.iat,
        exp: claims.exp,
        aud: claims.aud,
        iss: claims.iss,
        sub: claims.sub,
        role,
    })
}

pub async fn graphql_handler(
    State(ApiState {
        schema,
        settings,
        auth,
        ..
    }): State<ApiState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    // the graph router admitted the caller and signed a passport; it also forwards their token
    if let Some(passport) = headers.get(PASSPORT_HEADER) {
        let Some(auth) = auth.as_ref() else {
            return Ok(unauthorized("graph passports are not accepted here"));
        };
        let claims = match passport_claims(auth, passport.to_str()?).await {
            Ok(claims) => claims,
            Err(e) => return Ok(unauthorized(e.to_string())),
        };
        return Ok(schema
            .execute(req.into_inner().data(ValidatedClaims(claims)))
            .await
            .into());
    }

    let token = match headers.get("Authorization") {
        Some(auth_header) => Some(auth_header.to_str()?.replace("Bearer ", "")),
        None => None,
    };

    let Some(token) = token else {
        if cfg!(debug_assertions) {
            return Ok(schema.execute(req.into_inner()).await.into());
        }

        return Ok(unauthorized("Unauthorized request"));
    };

    let claims = tracing::info_span!("auth.validate_jwt")
        .in_scope(|| jwt::verify_jwt(settings.auth_secret.as_bytes(), &token))?;

    if cfg!(debug_assertions) {
        tracing::info!("verified token with claims: {:?}", claims);
    }

    Ok(schema
        .execute(
            req.into_inner()
                .data(ValidatedToken(token))
                .data(ValidatedClaims(claims)),
        )
        .await
        .into())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    recommendations: bool,
}

pub async fn health(
    State(ApiState { schema, .. }): State<ApiState>,
) -> Result<impl IntoResponse, AppError> {
    let request = async_graphql::Request::new(
        r#"
      {
        maccasHealth {
          recommendations
        }
      }
      "#,
    )
    .data(Untraced);

    let response = schema.execute(request).await;
    let health_response = serde_json::from_value::<HealthResponse>(
        response
            .data
            .into_json()?
            .get("maccasHealth")
            .context("can't find health in response")?
            .clone(),
    )?;

    if health_response.recommendations {
        Ok((StatusCode::OK, Json(health_response)))
    } else {
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json(health_response)))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SelfHealthResponse {
    status: String,
}

pub async fn self_health(
    State(ApiState { .. }): State<ApiState>,
) -> Result<Json<SelfHealthResponse>, AppError> {
    Ok(Json(SelfHealthResponse {
        status: "ok".to_string(),
    }))
}
