use crate::constants::{X_LOG_USER_ID, X_LOG_USER_NAME};
use rocket::{
    outcome::Outcome,
    request::{self, FromRequest},
    Request,
};
use std::convert::Infallible;

pub struct LogHeader {
    pub user_name: Option<String>,
    pub user_id: Option<String>,
    pub is_available: bool,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LogHeader {
    type Error = Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let user_id = request.headers().get_one(X_LOG_USER_ID);
        let user_name = request.headers().get_one(X_LOG_USER_NAME);

        if let (Some(user_id), Some(user_name)) = (user_id, user_name) {
            Outcome::Success(LogHeader {
                user_id: Some(user_id.to_string()),
                user_name: Some(user_name.to_string()),
                is_available: true,
            })
        } else {
            Outcome::Success(LogHeader {
                user_id: None,
                user_name: None,
                is_available: false,
            })
        }
    }
}
