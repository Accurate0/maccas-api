use self::types::{AddOfferInput, AddOfferResponse, RemoveOfferInput};
use crate::{graphql::ValidatedToken, settings::Settings};
use async_graphql::{Context, Object};
use event::{CreateEvent, CreateEventResponse, Event};
use reqwest::{header::AUTHORIZATION, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::prelude::Uuid;
use std::time::Duration;

mod types;

#[derive(Default)]
pub struct OffersMutation;

#[Object]
impl OffersMutation {
    async fn add_offer<'a>(
        &self,
        ctx: &Context<'a>,
        _input: AddOfferInput,
    ) -> async_graphql::Result<AddOfferResponse> {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let offer_id = Uuid::new_v4();
        let token = ctx.data_opt::<ValidatedToken>().map(|v| &v.0);

        let http_client = ctx.data::<ClientWithMiddleware>()?;
        let settings = ctx.data::<Settings>()?;

        let cleanup_event = CreateEvent {
            event: Event::Cleanup { offer_id },
            delay: Duration::from_secs(900),
        };

        let request_url = format!("{}/{}", settings.event_api_base, CreateEvent::path());
        let request = http_client.post(request_url).json(&cleanup_event);

        let request = if let Some(token) = token {
            request.header(AUTHORIZATION, format!("Bearer {token}"))
        } else {
            request
        };

        let response = request.send().await;

        match response {
            Ok(response) => match response.status() {
                StatusCode::CREATED => {
                    let id = response.json::<CreateEventResponse>().await?.id;
                    tracing::info!("created cleanup event with id {} created", id);
                }
                status => {
                    tracing::warn!("event failed with {} - {}", status, response.text().await?);
                }
            },
            Err(e) => tracing::warn!("event request failed with {}", e),
        }

        Ok(AddOfferResponse {
            id: offer_id,
            code: "1111".into(),
        })
    }

    async fn remove_offer<'a>(
        &self,
        _ctx: &Context<'a>,
        input: RemoveOfferInput,
    ) -> async_graphql::Result<Uuid> {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(input.id)
    }
}
