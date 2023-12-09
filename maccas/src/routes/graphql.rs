use super::Context;
use crate::graphql::query::Query;
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use rocket::{response::content::RawHtml, State};

pub type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

#[get("/playground")]
pub fn playground() -> RawHtml<String> {
    juniper_rocket::playground_source("/graphql", None)
}

#[get("/graphql?<request..>")]
pub async fn get_graphql(
    ctx: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, ctx).await
}

#[post("/graphql", data = "<request>")]
pub async fn post_graphql(
    ctx: &State<Context>,
    request: juniper_rocket::GraphQLRequest,
    schema: &State<Schema>,
) -> juniper_rocket::GraphQLResponse {
    request.execute(schema, ctx).await
}
