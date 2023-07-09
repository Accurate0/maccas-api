use crate::{constants::config::MAX_PROXY_COUNT, rng::RNG, types::config::ProxyConfig};
use rand::Rng;

pub async fn get_proxy(config: &ProxyConfig) -> reqwest::Proxy {
    let mut rng = RNG.lock().await;
    let random_number = rng.gen_range(1..=MAX_PROXY_COUNT);

    let username = format!("{}-{}", &config.username, random_number);
    log::info!("using proxy: {}", username);
    reqwest::Proxy::all(config.address.clone())
        .unwrap()
        .basic_auth(&username, &config.password)
}

pub fn get_specific_proxy(config: &ProxyConfig, random_number: i8) -> reqwest::Proxy {
    let username = format!("{}-{}", &config.username, random_number);
    log::info!("using proxy: {}", username);
    reqwest::Proxy::all(config.address.clone())
        .unwrap()
        .basic_auth(&username, &config.password)
}
