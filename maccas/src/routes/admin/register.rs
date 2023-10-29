use crate::{
    constants::config::{IMAGE_CDN, WEBSITE_BASE_URL},
    guards::admin::AdminOnlyRoute,
    routes,
    types::{api::RegistrationTokenResponse, error::ApiError, role::UserRole},
};
use aws_sdk_s3::primitives::ByteStream;
use foundation::aws;
use qrcode_generator::QrCodeEcc;
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Token that can be used for registration", body = RegistrationTokenResponse),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
)]
#[post("/admin/auth/register?<role>")]
pub async fn registration_token(
    ctx: &State<routes::Context<'_>>,
    _admin: AdminOnlyRoute,
    role: UserRole,
) -> Result<Json<RegistrationTokenResponse>, ApiError> {
    let registration_token = uuid::Uuid::new_v4().as_hyphenated().to_string();

    ctx.database
        .create_registration_token(&registration_token, role)
        .await?;

    let shared_config = aws::config::get_shared_config().await;
    let s3_client = aws_sdk_s3::Client::new(&shared_config);

    let subpath = format!("qr/{}.png", registration_token);
    let image_link = format!("{}/{subpath}", IMAGE_CDN);
    let link_in_qr_code = format!("{}/register?token={}", WEBSITE_BASE_URL, registration_token);

    let result: Vec<u8> =
        qrcode_generator::to_png_to_vec(link_in_qr_code, QrCodeEcc::Low, 1024).unwrap();

    s3_client
        .put_object()
        .bucket(&ctx.config.images.bucket_name)
        .key(subpath)
        .content_type("image/png")
        .body(ByteStream::from(result))
        .send()
        .await?;

    Ok(Json(RegistrationTokenResponse {
        token: registration_token,
        qr_code_link: image_link,
    }))
}
