use crate::constants::config::TOKEN_COOKIE_NAME;
use rocket::http::{Cookie, SameSite};

pub fn generate_token_cookie<'a>(value: String) -> Cookie<'a> {
    Cookie::build((TOKEN_COOKIE_NAME, value))
        .http_only(true)
        .same_site(SameSite::Strict)
        .build()
}
