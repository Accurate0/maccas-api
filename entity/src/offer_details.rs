//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "offer_details")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub proposition_id: i64,
    pub name: String,
    pub description: String,
    #[sea_orm(column_type = "Double")]
    pub price: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
