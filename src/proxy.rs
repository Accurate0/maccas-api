use crate::types::config::GeneralConfig;

pub fn get_proxy(config: &GeneralConfig) -> reqwest::Proxy {
    reqwest::Proxy::all(config.refresh.proxy.address.clone())
        .unwrap()
        .basic_auth(
            &config.refresh.proxy.username,
            &config.refresh.proxy.password,
        )
}
