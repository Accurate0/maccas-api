use rocket::{http::Status, Request};

#[catch(404)]
pub fn not_found(req: &Request) -> &'static str {
    log::warn!("response code: {} for request {:#?}", 404, req);
    ""
}

#[catch(500)]
pub fn internal_server_error(req: &Request) -> &'static str {
    log::warn!("response code: {} for request {:#?}", 500, req);
    ""
}

#[catch(401)]
pub fn not_authenticated(req: &Request) -> &'static str {
    log::warn!("response code: {} for request {:#?}", 401, req);
    ""
}

#[catch(default)]
pub fn default(status: Status, req: &Request) -> &'static str {
    log::warn!(
        "[default] response code: {:#?} for request {:#?}",
        status,
        req
    );
    ""
}
