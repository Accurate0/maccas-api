use api::{CreateEvent, CreateEventResponse};
use reqwest_middleware::{ClientWithMiddleware, reqwest::StatusCode};

use crate::settings::Settings;

pub async fn send_event(
    event: CreateEvent,
    settings: &Settings,
    http_client: &ClientWithMiddleware,
) -> anyhow::Result<()> {
    let token = base::jwt::generate_internal_jwt(
        settings.auth_secret.as_bytes(),
        "Maccas Scheduler",
        "Maccas Event",
    )?;

    let request_url = format!("{}/{}", settings.event_api_base, api::CreateEvent::path());

    let request = http_client
        .post(&request_url)
        .json(&event)
        .bearer_auth(token);

    let response = request.send().await;

    match response {
        Ok(response) => match response.status() {
            StatusCode::CREATED | StatusCode::OK => {
                let id = response.json::<CreateEventResponse>().await?.id;
                tracing::info!("created events with id {:?}", id);
            }
            status => {
                tracing::warn!("event failed with {} - {}", status, response.text().await?);
            }
        },
        Err(e) => tracing::warn!("event request failed with {}", e),
    }

    Ok(())
}
