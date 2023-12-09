use self::types::User;
use crate::{
    routes::Context,
    types::{role::UserRole, token::JwtClaim, user::UserOptions},
};
use async_graphql::Object;

mod types;

#[derive(Default)]
pub struct UserQuery;

#[Object]
impl UserQuery {
    async fn user<'a>(&self, gql_ctx: &async_graphql::Context<'a>) -> Result<User, anyhow::Error> {
        let claims = gql_ctx.data_unchecked::<JwtClaim>();

        Ok(User {
            id: claims.oid.clone(),
        })
    }
}

#[Object]
impl User {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn config<'ctx>(
        &self,
        gql_ctx: &async_graphql::Context<'ctx>,
    ) -> Result<UserOptions, anyhow::Error> {
        let ctx = gql_ctx.data_unchecked::<Context>();
        let claims = gql_ctx.data_unchecked::<JwtClaim>();

        Ok(ctx
            .database
            .user_repository
            .get_config_by_user_id(&claims.oid)
            .await?
            .into())
    }

    async fn role<'ctx>(
        &self,
        gql_ctx: &async_graphql::Context<'ctx>,
    ) -> Result<UserRole, anyhow::Error> {
        let ctx = gql_ctx.data_unchecked::<Context>();
        let claims = gql_ctx.data_unchecked::<JwtClaim>();

        ctx.database
            .user_repository
            .get_role(claims.username.clone())
            .await
    }
}
