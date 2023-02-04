use crate::{
    constants::{
        config::MAX_PROXY_COUNT,
        mc_donalds::default::{FILTER, STORE_UNIQUE_ID_TYPE},
    },
    proxy, routes,
    types::{error::ApiError, user::UserOptions},
};
use foundation::rocket::guards::authorization::RequiredAuthorizationHeader;
use rand::{rngs::StdRng, Rng, SeedableRng};
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
    let user_id = auth.claims.oid;
    match ctx.database.get_config_by_user_id(&user_id).await {
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
    let user_id = auth.claims.oid;
    let user_name = auth.claims.name;

    let mut rng = StdRng::from_entropy();
    let random_number = rng.gen_range(1..=MAX_PROXY_COUNT);

    let proxy = proxy::get_proxy(&ctx.config, random_number);
    let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
    let account = &ctx.config.mcdonalds.service_account;
    let account = &account.into();
    let api_client = ctx
        .database
        .get_specific_client(
            http_client,
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
            .set_config_by_user_id(&user_id, &config.into(), &user_name)
            .await?;
        Ok(Status::NoContent)
    } else {
        Ok(Status::BadRequest)
    }
}
