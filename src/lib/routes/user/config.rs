use crate::{
    guards::authorization::RequiredAuthorizationHeader,
    routes,
    types::{error::ApiError, jwt::JwtClaim, user::UserOptions},
};
use jwt::{Header, Token};
use rocket::{http::Status, serde::json::Json, State};

#[utoipa::path(
    get,
    path = "/user/config",
    responses(
        (status = 200, description = "Config for current user", body = UserOptions),
        (status = 404, description = "No config for this user"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "config",
)]
#[get("/user/config")]
pub async fn get_user_config(
    ctx: &State<routes::Context<'_>>,
    auth: RequiredAuthorizationHeader,
) -> Result<Json<UserOptions>, ApiError> {
    let value = auth.0.as_str().replace("Bearer ", "");
    let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value)?;
    let user_id = &jwt.claims().oid;

    match ctx.database.get_config_by_user_id(user_id).await {
        Ok(config) => Ok(Json(config)),
        Err(_) => Err(ApiError::NotFound),
    }
}

#[utoipa::path(
    post,
    path = "/user/config",
    request_body(
        content = UserOptions,
        content_type = "application/json",
    ),
    responses(
        (status = 204, description = "Updated/created config"),
        (status = 400, description = "Invalid configuration format"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "config",
)]
#[post("/user/config", data = "<config>")]
pub async fn update_user_config(
    ctx: &State<routes::Context<'_>>,
    auth: RequiredAuthorizationHeader,
    config: Json<UserOptions>,
) -> Result<Status, ApiError> {
    let value = auth.0.as_str().replace("Bearer ", "");
    let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value)?;
    let user_id = &jwt.claims().oid;
    let user_name = &jwt.claims().name;

    ctx.database.set_config_by_user_id(user_id, &config, user_name).await?;
    Ok(Status::NoContent)
}
