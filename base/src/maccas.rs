use crate::{constants, http::get_proxied_maccas_http_client};
use entity::accounts;
use reqwest::Proxy;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel, Set};

pub async fn get_activated_maccas_api_client(
    account: accounts::Model,
    proxy: Proxy,
    client_id: &str,
    db: &DatabaseConnection,
) -> Result<libmaccas::ApiClient, anyhow::Error> {
    let mut api_client = libmaccas::ApiClient::new(
        constants::mc_donalds::BASE_URL.to_owned(),
        get_proxied_maccas_http_client(proxy)?,
        client_id.to_owned(),
    );

    let now = chrono::Utc::now().naive_utc();

    // refresh case
    if (now - account.refreshed_at).num_minutes() >= 14 {
        api_client.set_auth_token(&account.access_token);
        let response = api_client
            .customer_login_refresh(&account.refresh_token)
            .await?;

        let response = response
            .body
            .response
            .ok_or_else(|| anyhow::Error::msg("access token refresh failed"))?;

        api_client.set_auth_token(&response.access_token);

        let mut update_model = account.into_active_model();

        update_model.access_token = Set(response.access_token);
        update_model.refresh_token = Set(response.refresh_token);
        tracing::info!("new tokens fetched, updating database");

        update_model.update(db).await?;
    } else {
        api_client.set_auth_token(&account.access_token);
    }

    Ok(api_client)
}
