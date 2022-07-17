use jwt::{Header, Token};
use lambda_http::Request;

use crate::{routes::Context, types::jwt::JwtClaim};

pub fn check_route_auth(path: &str, context: &Context, request: &Request) -> bool {
    let allowed_users = &context.config.routes.allowed_user_ids;

    let auth_header = request.headers().get(http::header::AUTHORIZATION);
    if let Some(auth_header) = auth_header {
        let value = auth_header.to_str().unwrap().replace("Bearer ", "");
        let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value).unwrap();

        if allowed_users.iter().any(|user_id| *user_id == jwt.claims().oid) {
            log::info!(
                "user {}/{} allowed access to {}",
                jwt.claims().oid,
                jwt.claims().name,
                path
            );
            return true;
        }

        return false;
    }

    // can't get to the api without header or key
    // if no header, assume key and allow
    true
}
