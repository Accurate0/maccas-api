use crate::{graphql::schema::MaccasSchema, types::error::ApiError};
use rocket::State;

#[utoipa::path(
    responses(
        (status = 200, description = "GraphQL Schema", body = String),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "docs",
)]
#[get("/docs/graphql")]
pub fn get_graphql(schema: &State<MaccasSchema>) -> Result<String, ApiError> {
    Ok(schema.sdl())
}
