use crate::{ConversionError, Database};
use entity::points::Model as PointsModel;
use sea_orm::prelude::Uuid;

impl Database<PointsModel> {
    pub fn convert_points_response(
        points: &libmaccas::types::response::PointInformationResponse,
        account_id: Uuid,
    ) -> Result<Self, ConversionError> {
        let now = chrono::offset::Utc::now().naive_utc();

        Ok(Database(PointsModel {
            account_id,
            current_points: points.total_points,
            lifetime_points: points.life_time_points,
            created_at: now,
            updated_at: now,
        }))
    }
}
