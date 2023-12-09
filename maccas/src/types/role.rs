use async_graphql::Enum;
use rocket::FromFormField;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(FromFormField, ToSchema, Enum, Copy)]
pub enum UserRole {
    Admin,
    Privileged,
    #[default]
    None,
}

impl UserRole {
    pub fn is_allowed_protected_access(&self) -> bool {
        matches!(self, UserRole::Admin | UserRole::Privileged)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, UserRole::Admin)
    }
}
