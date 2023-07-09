use crate::types::error::ApiError;
use rocket::http::Status;

#[utoipa::path(
    responses(
        (status = 204, description = "Server is healthy"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "health",
)]
#[get("/health/status")]
pub async fn get_status() -> Result<Status, ApiError> {
    Ok(Status::NoContent)
}
