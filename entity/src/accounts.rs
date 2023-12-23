//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    pub password: String,
    pub access_token: String,
    pub refresh_token: String,
    pub device_id: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::offers::Entity")]
    Offers,
    #[sea_orm(has_many = "super::points::Entity")]
    Points,
}

impl Related<super::offers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Offers.def()
    }
}

impl Related<super::points::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Points.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
