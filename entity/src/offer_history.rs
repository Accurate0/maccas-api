//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "offer_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub offer_id: i64,
    pub valid_from: DateTime,
    pub valid_to: DateTime,
    pub creation_date: DateTime,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub offer_proposition_id: i64,
    pub account_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::accounts::Entity",
        from = "Column::AccountId",
        to = "super::accounts::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Accounts,
    #[sea_orm(
        belongs_to = "super::offer_details::Entity",
        from = "Column::OfferPropositionId",
        to = "super::offer_details::Column::PropositionId",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    OfferDetails,
}

impl Related<super::accounts::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Accounts.def()
    }
}

impl Related<super::offer_details::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::OfferDetails.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
