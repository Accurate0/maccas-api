use crate::{doc::openapi::ApiDoc, types::error::ApiError};
use foundation::constants;
use rocket::serde::json::Json;
use utoipa::{
    openapi::{
        self,
        security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
        InfoBuilder, Server,
    },
    OpenApi,
};

#[get("/docs/openapi")]
pub fn get_openapi() -> Result<Json<openapi::OpenApi>, ApiError> {
    let mut spec = ApiDoc::openapi();
    let info = InfoBuilder::new().title("Maccas API").version("v1");
    spec.servers = Some(vec![Server::new(constants::MACCAS_API_BASE_URL)]);
    spec.info = info.build();

    let jwt = SecurityScheme::Http(
        HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .bearer_format("JWT")
            .build(),
    );

    let api_key = SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-Api-Key")));

    let components = &mut spec.components.as_mut().unwrap();
    components.add_security_scheme("JWT", jwt);
    components.add_security_scheme("API Key", api_key);

    Ok(Json(spec))
}
