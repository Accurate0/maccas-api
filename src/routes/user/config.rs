use crate::{
    constants::mc_donalds::default::{FILTER, STORE_UNIQUE_ID_TYPE},
    guards::authorization::RequiredAuthorizationHeader,
    retry::wrap_in_middleware,
    routes,
    types::{error::ApiError, jwt::JwtClaim, user::UserOptions},
};
use jwt::{Header, Token};
use rocket::{http::Status, serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Config for current user", body = UserOptions),
        (status = 404, description = "No config for this user"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "user",
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
        Ok(config) => Ok(Json(config.into())),
        Err(_) => Err(ApiError::NotFound),
    }
}

#[utoipa::path(
    request_body(
        content = UserOptions,
        content_type = "application/json",
    ),
    responses(
        (status = 204, description = "Updated/created config"),
        (status = 400, description = "Invalid configuration format"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "user",
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

    let http_client = foundation::http::get_http_client(wrap_in_middleware);
    let account = &ctx.config.mcdonalds.service_account;
    let account = &account.into();
    let api_client = ctx
        .database
        .get_specific_client(
            &http_client,
            &ctx.config.mcdonalds.client_id,
            &ctx.config.mcdonalds.client_secret,
            &ctx.config.mcdonalds.sensor_data,
            account,
            false,
        )
        .await?;

    let resp = api_client
        .get_restaurant(&config.store_id, FILTER, STORE_UNIQUE_ID_TYPE)
        .await?;

    if resp.status.is_success() {
        // ensure the store id exists
        // override the name
        let response = resp.body.response;
        let store_name = match response {
            Some(response) => response.restaurant.name,
            None => "Unknown/Invalid Name".to_owned(),
        };

        let config = UserOptions {
            store_name: Some(store_name),
            ..config.0
        };

        ctx.database
            .set_config_by_user_id(user_id, &config.into(), user_name)
            .await?;
        Ok(Status::NoContent)
    } else {
        Ok(Status::BadRequest)
    }
}
