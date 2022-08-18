use crate::constants::mc_donalds;
use crate::constants::mc_donalds::default::{FILTER, STORE_UNIQUE_ID_TYPE};
use crate::guards::authorization::AuthorizationHeader;
use crate::guards::correlation_id::CorrelationId;
use crate::guards::log::LogHeader;
use crate::logging::log_external;
use crate::types::api::OfferResponse;
use crate::types::error::ApiError;
use crate::types::jwt::JwtClaim;
use crate::webhook::execute::execute_discord_webhooks;
use crate::{client, routes};
use anyhow::Context;
use chrono::Duration;
use jwt::{Header, Token};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Added a deal", body = OfferResponse),
        (status = 400, description = "Error on McDonald's side"),
        (status = 404, description = "Deal not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    params(
        ("x-log-user-id" = Option<String>, header, description = "The user id to log for"),
        ("x-log-user-name" = Option<String>, header, description = "The user name to log for"),
    ),
    tag = "deals",
)]
#[post("/deals/<deal_id>?<store>")]
pub async fn add_deal(
    ctx: &State<routes::Context<'_>>,
    deal_id: &str,
    store: Option<i64>,
    auth: AuthorizationHeader,
    log: LogHeader,
    correlation_id: CorrelationId,
) -> Result<Json<OfferResponse>, ApiError> {
    if let Ok((account, offer)) = ctx.database.get_offer_by_id(deal_id).await {
        let http_client = client::get_http_client();
        let api_client = ctx
            .database
            .get_specific_client(
                &http_client,
                &ctx.config.client_id,
                &ctx.config.client_secret,
                &ctx.config.sensor_data,
                &account,
                false,
            )
            .await?;

        let offer_id = offer.offer_id;
        let offer_proposition_id = offer.offer_proposition_id.to_string();

        let current_deal_stack = api_client
            .get_offers_dealstack(
                mc_donalds::default::OFFSET,
                &store.unwrap_or(mc_donalds::default::STORE_ID),
            )
            .await?
            .body
            .response
            .context("no deal stack response")?
            .deal_stack;

        if let Some(current_deal_stack) = current_deal_stack {
            if current_deal_stack.len() != 1
                || !current_deal_stack.into_iter().any(|deal| {
                    deal.offer_id == offer_id && deal.offer_proposition_id == offer_proposition_id
                })
            {
                return Err(ApiError::AccountNotAvailable);
            }
        }

        // lock the deal from appearing in GET /deals
        ctx.database.lock_deal(deal_id, Duration::hours(12)).await?;

        let resp = api_client
            .add_to_offers_dealstack(
                &offer_proposition_id,
                mc_donalds::default::OFFSET,
                &store.unwrap_or(mc_donalds::default::STORE_ID),
            )
            .await?;

        // this can cause the offer id to change.. for offers with id == 0
        // we need to update the database to avoid inconsistency
        // but we also need to find the uuid for the refreshed deal and lock it
        // only need to do this when the deal was successfully added, otherwise the refresh won't
        // get any new offer_id
        if resp.status.is_success() && offer_id == 0 {
            log::info!("offer_id = 0, refreshing account: {}", account);
            let mut new_offers = ctx
                .database
                .refresh_offer_cache_for(&account, &api_client, &ctx.config.ignored_offer_ids)
                .await?;

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

                    ctx.database
                        .set_offers_for(&account.account_name, &new_offers)
                        .await?;
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

        // if we get 409 Conflict. offer already exists
        let resp = if resp.status.as_u16() == 409 {
            api_client
                .get_offers_dealstack(
                    mc_donalds::default::OFFSET,
                    &store.unwrap_or(mc_donalds::default::STORE_ID),
                )
                .await?
        } else {
            // jwt has priority as it's more reliable
            let (user_id, user_name) = if let Some(auth_header) = auth.0 {
                let auth_header = auth_header.replace("Bearer ", "");
                let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&auth_header)?;
                let claims = jwt.claims();

                (Some(claims.oid.clone()), Some(claims.name.clone()))
            } else if log.is_available {
                (Some(log.user_id.unwrap()), Some(log.user_name.unwrap()))
            } else {
                (None, None)
            };

            if let (Some(user_id), Some(user_name)) = (user_id, user_name) {
                if !ctx
                    .config
                    .log
                    .ignored_user_ids
                    .iter()
                    .any(|ignored| *ignored == user_id)
                {
                    let restaurant_info = api_client
                        .get_restaurant(
                            &store.unwrap_or(mc_donalds::default::STORE_ID),
                            FILTER,
                            STORE_UNIQUE_ID_TYPE,
                        )
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

                    let log_fut = log_external(
                        &http_client,
                        &ctx.config,
                        &user_id,
                        &user_name,
                        &offer,
                        &correlation_id.0,
                    );

                    let discord_webhook_fut = execute_discord_webhooks(
                        &http_client,
                        &ctx.config,
                        &user_name,
                        &offer,
                        &store_name,
                    );

                    tokio::join!(log_fut, discord_webhook_fut);
                }
            }
            resp
        };

        let resp = OfferResponse::from(resp.body);
        Ok(Json(resp))
    } else {
        Err(ApiError::NotFound)
    }
}
