use self::types::{AddOfferInput, AddOfferResponse, RemoveOfferInput};
use crate::{
    graphql::{ValidatedClaims, ValidatedToken},
    settings::Settings,
};
use async_graphql::{Context, Object};
use entity::sea_orm_active_enums::Action;
use event::{CreateEvent, CreateEventResponse, Event};
use reqwest::{header::AUTHORIZATION, StatusCode};
use reqwest_middleware::ClientWithMiddleware;
use sea_orm::{prelude::Uuid, ActiveModelTrait, DatabaseConnection, Set};
use std::time::Duration;

mod types;

#[derive(Default)]
pub struct OffersMutation;

#[Object]
impl OffersMutation {
    async fn add_offer<'a>(
        &self,
        ctx: &Context<'a>,
        input: AddOfferInput,
    ) -> async_graphql::Result<AddOfferResponse> {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let db = ctx.data::<DatabaseConnection>()?;
        let claims = ctx.data_opt::<ValidatedClaims>();
        let fake_offer_id = Uuid::new_v4();
        let validated_proposition_id = input.offer_proposition_id;
        let token = ctx.data_opt::<ValidatedToken>().map(|v| &v.0);

        let http_client = ctx.data::<ClientWithMiddleware>()?;
        let settings = ctx.data::<Settings>()?;

        let transaction_id = Uuid::new_v4();
        if let Some(claims) = claims {
            entity::offer_audit::ActiveModel {
                action: Set(Action::Add),
                proposition_id: Set(validated_proposition_id),
                user_id: Set(claims.0.user_id.parse()?),
                transaction_id: Set(transaction_id),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        let cleanup_event = CreateEvent {
            event: Event::Cleanup {
                offer_id: fake_offer_id,
                transaction_id,
            },
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
            id: fake_offer_id,
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
