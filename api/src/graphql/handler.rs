use crate::auth::{AuthError, Authenticator};
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

async fn kanidm_claims(auth: &Authenticator, token: &str) -> Result<JwtClaims, AuthError> {
    let claims = auth.authorize(token).await?;

    Ok(JwtClaims {
        user_id: String::new(),
        session_id: None,
        iat: claims.iat,
        exp: claims.exp,
        aud: claims.aud,
        iss: claims.iss,
        sub: claims.sub,
        role: vec![Role::Admin, Role::Points],
    })
}

// FIXME: tracing the authorization code
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

    let claims = match auth.as_ref().filter(|auth| auth.issued(&token)) {
        Some(auth) => match kanidm_claims(auth, &token).await {
            Ok(claims) => claims,
            Err(e) => return Ok(unauthorized(e.to_string())),
        },
        None => jwt::verify_jwt(settings.auth_secret.as_bytes(), &token)?,
    };

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
    );

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
