use crate::database::refresh::RefreshRepository;
use crate::types::api::LastRefreshInformation;
use crate::types::error::ApiError;
use rocket::serde::json::Json;
use rocket::State;

#[utoipa::path(
    responses(
        (status = 200, description = "Last Refresh of Cache", body = LastRefreshInformation),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "deals",
)]
#[get("/deals/last-refresh")]
pub async fn get_last_refresh(
    refresh_repo: &State<RefreshRepository>,
) -> Result<Json<LastRefreshInformation>, ApiError> {
    let response = refresh_repo.get_last_refresh().await?;

    Ok(Json(LastRefreshInformation {
        last_refresh: response,
    }))
}
