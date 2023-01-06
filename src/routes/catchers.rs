use rocket::{http::Status, Request};

#[catch(default)]
pub fn default(status: Status, req: &Request) -> &'static str {
    log::warn!("[default] response code: {} for request {}", status, req);
    ""
}
