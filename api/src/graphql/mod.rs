use self::queries::{health::HealthQuery, offers::OffersQuery, points::PointsQuery};
use crate::types::{ApiState, AppError, JwtClaims};
use async_graphql::{
    http::GraphiQLSource, EmptyMutation, EmptySubscription, MergedObject, Schema, ServerError,
};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{debug_handler, extract::State, http::HeaderMap, response::IntoResponse};
use hmac::{Hmac, Mac};
use jwt::{Header, Token, VerifyWithKey};
use sha2::Sha256;

pub mod queries;

#[derive(Default, MergedObject)]
pub struct QueryRoot(HealthQuery, OffersQuery, PointsQuery);

pub type MaccasSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

pub async fn graphiql() -> impl IntoResponse {
    axum::response::Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[debug_handler]
pub async fn graphql_handler(
    State(ApiState { schema, settings }): State<ApiState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> Result<GraphQLResponse, AppError> {
    let auth_header = headers.get("Authorization");
    if cfg!(debug_assertions) {
        if let Some(auth_header) = auth_header {
            let key: Hmac<Sha256> = Hmac::new_from_slice(settings.auth_secret.as_bytes())?;
            let auth_header = auth_header.to_str()?.replace("Bearer ", "");
            let unverified: Token<Header, JwtClaims, jwt::Unverified<'_>> =
                Token::parse_unverified(&auth_header)?;
            let token: Token<_, _, jwt::Verified> = unverified.verify_with_key(&key)?;
            tracing::info!("verified token with claims: {:?}", token.claims());
        }

        return Ok(schema.execute(req.into_inner()).await.into());
    }

    match auth_header {
        Some(auth_header) => {
            let key: Hmac<Sha256> = Hmac::new_from_slice(settings.auth_secret.as_bytes())?;
            let auth_header = auth_header.to_str()?.replace("Bearer ", "");
            let unverified: Token<Header, JwtClaims, jwt::Unverified<'_>> =
                Token::parse_unverified(&auth_header)?;
            let _token: Token<_, _, jwt::Verified> = unverified.verify_with_key(&key)?;

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
