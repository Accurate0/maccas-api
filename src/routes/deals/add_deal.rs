use crate::constants::mc_donalds::default::{FILTER, STORE_UNIQUE_ID_TYPE};
use crate::constants::{mc_donalds, DEFAULT_LOCK_TTL_HOURS};
use crate::database::types::AuditActionType;
use crate::guards::authorization::AuthorizationHeader;
use crate::routes;
use crate::types::api::OfferResponse;
use crate::types::error::ApiError;
use crate::types::images::OfferImageBaseName;
use crate::types::sqs::{CleanupMessage, ImagesRefreshMessage};
use crate::webhook::execute::execute_discord_webhooks;
use anyhow::Context;
use chrono::Duration;
use foundation::types::jwt::JwtClaim;
use itertools::Itertools;
use jwt::{Header, Token};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Added a deal", body = OfferResponse),
        (status = 400, description = "Error on McDonald's side"),
        (status = 404, description = "Deal not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "deals",
)]
#[post("/deals/<deal_id>?<store>")]
pub async fn add_deal(
    ctx: &State<routes::Context<'_>>,
    deal_id: &str,
    store: i64,
    auth: AuthorizationHeader,
) -> Result<Json<OfferResponse>, ApiError> {
    if let Ok((account, offer)) = ctx.database.get_offer_by_id(deal_id).await {
        let http_client = foundation::http::get_default_http_client();
        let api_client = ctx
            .database
            .get_specific_client(
                &http_client,
                &ctx.config.mcdonalds.client_id,
                &ctx.config.mcdonalds.client_secret,
                &ctx.config.mcdonalds.sensor_data,
                &account,
                false,
            )
            .await?;

        let offer_id = offer.offer_id;
        let offer_proposition_id = offer.offer_proposition_id.to_string();

        let current_deal_stack = api_client
            .get_offers_dealstack(mc_donalds::default::OFFSET, &store)
            .await?
            .body
            .response
            .context("no deal stack response")?
            .deal_stack;

        if let Some(current_deal_stack) = current_deal_stack {
            if !current_deal_stack.into_iter().all(|deal| {
                deal.offer_id == offer_id && deal.offer_proposition_id == offer_proposition_id
            }) {
                return Err(ApiError::AccountNotAvailable);
            }
        }

        // lock the deal from appearing in GET /deals
        ctx.database
            .lock_deal(deal_id, Duration::hours(DEFAULT_LOCK_TTL_HOURS))
            .await?;

        let resp = api_client
            .add_to_offers_dealstack(&offer_proposition_id, mc_donalds::default::OFFSET, &store)
            .await?;

        // this can cause the offer id to change.. for offers with id == 0
        // we need to update the database to avoid inconsistency
        // but we also need to find the uuid for the refreshed deal and lock it
        // only need to do this when the deal was successfully added, otherwise the refresh won't
        // get any new offer_id
        if resp.status.is_success() && offer_id == 0 {
            // consider doing this async...
            // may not be quick enough...
            log::info!("offer_id = 0, refreshing account: {}", account);
            let mut new_offers = ctx
                .database
                .refresh_offer_cache_for(
                    &account,
                    &api_client,
                    &ctx.config.mcdonalds.ignored_offer_ids,
                )
                .await?;

            // this can uncover new deals, lets fetch the images for these
            if ctx.config.images.enabled {
                let image_base_names = new_offers
                    .iter()
                    .map(|offer| OfferImageBaseName {
                        original: offer.original_image_base_name.clone(),
                        new: offer.image_base_name.clone(),
                    })
                    .unique_by(|offer| offer.original.clone())
                    .collect();

                foundation::aws::sqs::send_to_queue(
                    &ctx.sqs_client,
                    &ctx.config.images.queue_name,
                    ImagesRefreshMessage { image_base_names },
                )
                .await?;
            }

            match new_offers
                .iter_mut()
                .find(|new_offer| **new_offer == offer)
                .context("must find current offer in new offers list")
            {
                Ok(new_matching_offer) => {
                    log::info!(
                        "located matching deal in after refresh: {}",
                        new_matching_offer.deal_uuid
                    );
                    // update the new offer with the old uuid
                    // no need to lock it anymore
                    new_matching_offer.deal_uuid = offer.deal_uuid.clone();

                    ctx.database.set_offers_for(&account, &new_offers).await?;
                    log::info!("updated uuid, and saved: {}", offer.deal_uuid);
                }
                Err(e) => {
                    // log error and dump information
                    // we can survive with this error
                    log::error!("failed to find matching offer in new offers list: {}", e);
                    log::error!("{:#?}", &offer);
                    log::error!("{:#?}", &new_offers);
                    log::error!("{:#?}", &account);
                    log::error!("{:#?}", &api_client);
                }
            };
        }

        // if adding to the deal stack fails, we fail...
        // we let the code above lock the deal though.
        // likely case is someone redeeming a deal but also removing it..
        // this lock will keep it removed and provide an error
        // 409 Conflict means the offer already exists
        // 404 when offer is already redeemed
        if !resp.status.is_success() && resp.status.as_u16() != 409 {
            return Err(ApiError::McDonaldsError);
        }

        let user_id = if let Some(auth_header) = auth.0 {
            let auth_header = auth_header.replace("Bearer ", "");
            let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&auth_header)?;
            let claims = jwt.claims();
            let user_name = &jwt.claims().name;
            let user_id = &jwt.claims().oid;

            if !claims.extension_role.is_admin() && ctx.config.api.discord_deal_use.enabled {
                let restaurant_info = api_client
                    .get_restaurant(&store, FILTER, STORE_UNIQUE_ID_TYPE)
                    .await;

                let store_name = match restaurant_info {
                    Ok(restaurant_info) => {
                        let response = restaurant_info.body.response;
                        match response {
                            Some(response) => response.restaurant.name,
                            None => "Unknown/Invalid Name".to_owned(),
                        }
                    }
                    _ => "Error getting store name".to_string(),
                };

                execute_discord_webhooks(&http_client, &ctx.config, user_name, &offer, &store_name)
                    .await;
            }

            ctx.database
                .add_to_audit(
                    AuditActionType::Add,
                    Some(user_id.to_string()),
                    Some(user_name.to_string()),
                    &offer,
                )
                .await;

            Some(user_id.to_owned())
        } else {
            ctx.database
                .add_to_audit(AuditActionType::Add, None, None, &offer)
                .await;
            None
        };

        // if we get 409 Conflict. offer already exists
        let resp = if resp.status.as_u16() == 409 {
            api_client
                .get_offers_dealstack(mc_donalds::default::OFFSET, &store)
                .await?
        } else {
            // queue this to be removed in 15 minutes
            if ctx.config.cleanup.enabled {
                foundation::aws::sqs::send_to_queue(
                    &ctx.sqs_client,
                    &ctx.config.cleanup.queue_name,
                    CleanupMessage {
                        user_id,
                        deal_uuid: deal_id.to_string(),
                        store_id: store,
                    },
                )
                .await?;
            }

            resp
        };

        let resp = OfferResponse::from(resp.body);
        Ok(Json(resp))
    } else {
        Err(ApiError::NotFound)
    }
}
