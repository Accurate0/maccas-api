//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "events")]
pub struct Model {
    pub name: String,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub event_id: Uuid,
    #[sea_orm(column_type = "JsonBinary")]
    pub data: Json,
    pub is_completed: bool,
    pub should_be_completed_at: DateTime,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub completed_at: Option<DateTime>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
