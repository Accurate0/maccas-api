use async_graphql::Object;
use entity::points;
use sea_orm::prelude::Uuid;

pub struct Points(pub points::Model);

#[Object]
impl Points {
    pub async fn account_id(&self) -> &Uuid {
        &self.0.account_id
    }

    pub async fn current_points(&self) -> &i64 {
        &self.0.current_points
    }

    pub async fn lifetime_points(&self) -> &i64 {
        &self.0.lifetime_points
    }
}
