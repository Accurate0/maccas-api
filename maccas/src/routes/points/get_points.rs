use crate::{
    database::point::PointRepository,
    guards::protected::ProtectedRoute,
    types::{api::AccountPointResponse, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "List of all account points", body = AccountPointResponse),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "points",
)]
#[get("/points")]
pub async fn get_points(
    _protected: ProtectedRoute,
    point_repo: &State<PointRepository>,
) -> Result<Json<AccountPointResponse>, ApiError> {
    let point_map = point_repo.get_point_map().await?;
    Ok(Json(point_map.into()))
}
