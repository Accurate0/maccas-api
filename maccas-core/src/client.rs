use crate::middleware;
use std::time::Duration;

pub fn get_http_client() -> reqwest_middleware::ClientWithMiddleware {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    middleware::get_middleware_http_client(client)
}

pub fn get_http_client_with_headers(
    headers: http::HeaderMap,
) -> reqwest_middleware::ClientWithMiddleware {
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(10))
        .default_headers(headers)
        .build()
        .unwrap();
    middleware::get_middleware_http_client(client)
}
