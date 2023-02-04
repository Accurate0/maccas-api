use crate::types::config::GeneralConfig;

pub fn get_proxy(config: &GeneralConfig, random_number: i8) -> reqwest::Proxy {
    let username = format!("{}-{}", &config.refresh.proxy.username, random_number);
    reqwest::Proxy::all(config.refresh.proxy.address.clone())
        .unwrap()
        .basic_auth(&username, &config.refresh.proxy.password)
}
