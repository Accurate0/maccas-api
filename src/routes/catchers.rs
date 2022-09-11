use rocket::Request;

#[catch(404)]
pub fn not_found(_req: &Request) -> &'static str {
    ""
}

#[catch(500)]
pub fn internal_server_error(_req: &Request) -> &'static str {
    ""
}
