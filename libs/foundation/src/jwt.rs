use aliri_oauth2::Authority;

pub async fn create_authority_with_defaults(
    jwks_url: impl ToString,
    allowed_audience: impl ToString,
) -> Result<Authority, anyhow::Error> {
    let validator = aliri::jwt::CoreValidator::default()
        .add_allowed_audience(aliri::jwt::Audience::from(allowed_audience.to_string()))
        .with_leeway_secs(10)
        .check_expiration()
        .check_not_before();

    Ok(aliri_oauth2::Authority::new_from_url(jwks_url.to_string(), validator).await?)
}
