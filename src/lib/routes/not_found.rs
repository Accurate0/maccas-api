use rocket::{http::Status, Request};

#[catch(404)]
pub fn not_found(_req: &Request) -> Status {
    Status::NotFound
}
