use async_graphql::Guard;
use base::jwt::Role;

use super::ValidatedClaims;

pub struct RoleGuard {
    role: Role,
}

impl RoleGuard {
    pub fn with_role(role: Role) -> Self {
        Self { role }
    }
}

impl Guard for RoleGuard {
    async fn check(&self, ctx: &async_graphql::Context<'_>) -> async_graphql::Result<()> {
        match ctx
            .data_opt::<ValidatedClaims>()
            .map(|c| &c.0.role)
            .is_some_and(|r| r.contains(&self.role))
        {
            true => Ok(()),
            false => Err(format!("Required role: {:?} is missing from token", self.role).into()),
        }
    }
}
