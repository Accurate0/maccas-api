use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
