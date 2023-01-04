use rand::Rng;

use crate::types::config::GeneralConfig;

pub fn get_proxy(config: &GeneralConfig) -> reqwest::Proxy {
    let mut rng = rand::thread_rng();
    let proxy_chosen = rng.gen_range(1..config.refresh.proxy.count);
    let username = format!("{}{}", config.refresh.proxy.username_prefix, proxy_chosen);
    log::info!("choosing proxy {:#?}", username);

    reqwest::Proxy::all(config.refresh.proxy.address.clone())
        .unwrap()
        .basic_auth(username.as_str(), &config.refresh.proxy.password)
}
