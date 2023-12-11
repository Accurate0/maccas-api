use crate::{
    graphql::schema::MaccasSchema, guards::required_authorization::RequiredAuthorizationHeader,
};
use async_graphql::http::GraphiQLSource;
use async_graphql_rocket::{GraphQLQuery, GraphQLRequest, GraphQLResponse};
use rocket::{
    response::content::{self},
    State,
};

#[rocket::get("/playground")]
pub fn graphiql() -> content::RawHtml<String> {
    content::RawHtml(GraphiQLSource::build().endpoint("/graphql").finish())
}

#[rocket::get("/graphql?<query..>")]
pub async fn graphql_query(schema: &State<MaccasSchema>, query: GraphQLQuery) -> GraphQLResponse {
    query.execute(schema.inner()).await
}

#[rocket::post("/graphql", data = "<request>", format = "application/json")]
#[tracing::instrument(skip(schema, auth, request))]
pub async fn graphql_request(
    schema: &State<MaccasSchema>,
    request: GraphQLRequest,
    auth: RequiredAuthorizationHeader,
) -> GraphQLResponse {
    tracing::info!("query: {}", request.0.query);
    request.data(auth.claims).execute(schema.inner()).await
}
