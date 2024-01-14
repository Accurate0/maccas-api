use crate::types::{ApiState, AppError};
use anyhow::Context;
use async_graphql::{http::GraphiQLSource, ServerError};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use base::jwt;

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub struct ValidatedToken(pub String);

pub async fn graphql_handler(
    State(ApiState { schema, settings }): State<ApiState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    let auth_header = headers.get("Authorization");
    if cfg!(debug_assertions) {
        let token = if let Some(auth_header) = auth_header {
            let token = &auth_header.to_str()?.replace("Bearer ", "");

            let claims = jwt::verify_jwt(settings.auth_secret.as_bytes(), token)?;
            tracing::info!("verified token with claims: {:?}", claims);

            Some(token.clone())
        } else {
            None
        };

        let req = if let Some(token) = token {
            req.into_inner().data(ValidatedToken(token))
        } else {
            req.into_inner()
        };

        return Ok(schema.execute(req).await.into());
    }

    match auth_header {
        Some(auth_header) => {
            let token = &auth_header.to_str()?.replace("Bearer ", "");

            jwt::verify_jwt(settings.auth_secret.as_bytes(), token)?;

            Ok(schema
                .execute(req.into_inner().data(ValidatedToken(token.to_string())))
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
}

pub async fn health(
    State(ApiState { schema, .. }): State<ApiState>,
) -> Result<Json<HealthResponse>, AppError> {
    let request = async_graphql::Request::new(
        r#"
      {
        health {
          database
          event
        }
      }
      "#,
    );

    let response = schema.execute(request).await;

    Ok(Json(serde_json::from_value(
        response
            .data
            .into_json()?
            .get("health")
            .context("can't find health in response")?
            .clone(),
    )?))
}
