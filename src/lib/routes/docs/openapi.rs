use crate::{constants::api_base, doc::openapi::ApiDoc, types::error::ApiError};
use rocket::serde::json::Json;
use utoipa::{
    openapi::{self, InfoBuilder, Server},
    OpenApi,
};

#[get("/docs/openapi")]
pub fn get_openapi() -> Result<Json<openapi::OpenApi>, ApiError> {
    let mut spec = ApiDoc::openapi();
    let info = InfoBuilder::new().title("Maccas API").version("v1");
    spec.servers = Some(vec![Server::new(api_base::THIS)]);
    spec.info = info.build();

    Ok(Json(spec))
}
