use crate::database::types::{AuditActionType, OfferDatabase};

#[derive(Debug)]
pub struct AuditEntry {
    pub action: AuditActionType,
    pub user_id: String,
    pub offer: OfferDatabase,
}
