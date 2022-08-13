use libapi::routes::docs::openapi::get_openapi;

fn main() {
    let openapi = get_openapi();
    println!("{}", openapi.unwrap().to_pretty_json().unwrap());
}
