use self::queries::{
    health::HealthQuery, locations::LocationsQuery, offers::OffersQuery, points::PointsQuery,
};
use crate::types::{ApiState, AppError};
use async_graphql::{
    http::GraphiQLSource, EmptyMutation, EmptySubscription, MergedObject, Schema, ServerError,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use base::jwt;

pub mod queries;

#[derive(Default, MergedObject)]
pub struct QueryRoot(HealthQuery, OffersQuery, PointsQuery, LocationsQuery);

pub type MaccasSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub async fn graphql_handler(
    State(ApiState { schema, settings }): State<ApiState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    let auth_header = headers.get("Authorization");
    if cfg!(debug_assertions) {
        if let Some(auth_header) = auth_header {
            let claims = jwt::verify_jwt(
                settings.auth_secret.as_bytes(),
                &auth_header.to_str()?.replace("Bearer ", ""),
            )?;
            tracing::info!("verified token with claims: {:?}", claims);
        }

        return Ok(schema.execute(req.into_inner()).await.into());
    }

    match auth_header {
        Some(auth_header) => {
            jwt::verify_jwt(
                settings.auth_secret.as_bytes(),
                &auth_header.to_str()?.replace("Bearer ", ""),
            )?;

            Ok(schema.execute(req.into_inner()).await.into())
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
