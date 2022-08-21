use crate::{
    guards::protected::ProtectedRoute,
    routes,
    types::{api::AccountPointResponse, error::ApiError},
};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "List of all account points", body = AccountPointResponse),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "points",
    params(
        ("X-Maccas-JWT-Bypass" = Option<String>, header, description = "Key to bypass JWT checks"),
    ),
)]
#[get("/points")]
pub async fn get_points(
    ctx: &State<routes::Context<'_>>,
    _protected: ProtectedRoute,
) -> Result<Json<AccountPointResponse>, ApiError> {
    let point_map = ctx.database.get_point_map().await?;
    Ok(Json(AccountPointResponse::from(point_map)))
}
