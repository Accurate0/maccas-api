use crate::types::{ApiState, AppError};
use anyhow::Context;
use async_graphql::{http::GraphiQLSource, ServerError};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::http::StatusCode;
use axum::response::Result;
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use base::jwt::{self, JwtClaims};

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/v1/graphql").finish()).into_response()
}

pub struct ValidatedToken(pub String);
pub struct ValidatedClaims(pub JwtClaims);

// FIXME: tracing the authorization code
pub async fn graphql_handler(
    State(ApiState { schema, settings }): State<ApiState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    let auth_header = headers.get("Authorization");
    let mut claims = None;

    if cfg!(debug_assertions) {
        let token = if let Some(auth_header) = auth_header {
            let token = &auth_header.to_str()?.replace("Bearer ", "");

            let validated_claims = jwt::verify_jwt(settings.auth_secret.as_bytes(), token)?;
            tracing::info!("verified token with claims: {:?}", validated_claims);

            claims = Some(validated_claims);
            Some(token.clone())
        } else {
            None
        };

        let req_inner = req.into_inner();
        let req = if let Some(claims) = claims {
            req_inner.data(ValidatedClaims(claims))
        } else {
            req_inner
        };

        let req = if let Some(token) = token {
            req.data(ValidatedToken(token))
        } else {
            req
        };

        return Ok(schema.execute(req).await.into());
    }

    match auth_header {
        Some(auth_header) => {
            let token = &auth_header.to_str()?.replace("Bearer ", "");

            let claims = jwt::verify_jwt(settings.auth_secret.as_bytes(), token)?;

            Ok(schema
                .execute(
                    req.into_inner()
                        .data(ValidatedToken(token.to_string()))
                        .data(ValidatedClaims(claims)),
                )
                .await
                .into())
        }
        None => {
            let mut response = async_graphql::Response::new(());
            response
                .errors
                .push(ServerError::new("Unauthorized request", None));

            Ok(GraphQLResponse::from(response))
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    database: bool,
    event: bool,
    batch: bool,
}

pub async fn health(
    State(ApiState { schema, .. }): State<ApiState>,
) -> Result<impl IntoResponse, AppError> {
    let request = async_graphql::Request::new(
        r#"
      {
        health {
          database
          event
          batch
        }
      }
      "#,
    );

    let response = schema.execute(request).await;
    let health_response = serde_json::from_value::<HealthResponse>(
        response
            .data
            .into_json()?
            .get("health")
            .context("can't find health in response")?
            .clone(),
    )?;

    if health_response.database && health_response.event && health_response.batch {
        Ok((StatusCode::OK, Json(health_response)))
    } else {
        Ok((StatusCode::SERVICE_UNAVAILABLE, Json(health_response)))
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SelfHealthResponse {
    database: bool,
}

pub async fn self_health(
    State(ApiState { schema, .. }): State<ApiState>,
) -> Result<Json<SelfHealthResponse>, AppError> {
    let request = async_graphql::Request::new(
        r#"
      {
        health {
          database
        }
      }
      "#,
    );

    let response = schema.execute(request).await;
    let health_response = serde_json::from_value::<SelfHealthResponse>(
        response
            .data
            .into_json()?
            .get("health")
            .context("can't find health in response")?
            .clone(),
    )?;

    if health_response.database {
        Ok(Json(health_response))
    } else {
        Err(AppError::StatusCode(StatusCode::SERVICE_UNAVAILABLE))
    }
}
