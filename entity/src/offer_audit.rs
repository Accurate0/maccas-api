//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.0

use super::sea_orm_active_enums::Action;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "offer_audit")]
pub struct Model {
    pub action: Action,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub transaction_id: Uuid,
    pub proposition_id: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub user_id: Option<Uuid>,
    pub migrated: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::offer_details::Entity",
        from = "Column::PropositionId",
        to = "super::offer_details::Column::PropositionId",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    OfferDetails,
}

impl Related<super::offer_details::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OfferDetails.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
