use crate::constants::config::DEFAULT_LOCK_TTL_HOURS;
use crate::constants::mc_donalds;
use crate::database::account::AccountRepository;
use crate::database::audit::AuditRepository;
use crate::database::offer::OfferRepository;
use crate::database::types::AuditActionType;
use crate::guards::required_authorization::RequiredAuthorizationHeader;
use crate::types::api::OfferResponse;
use crate::types::error::ApiError;
use crate::types::images::OfferImageBaseName;
use crate::types::sqs::{CleanupMessage, ImagesRefreshMessage};
use crate::{proxy, routes};
use anyhow::Context;
use chrono::Duration;
use foundation::either::Either;
use itertools::Itertools;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Added a deal", body = OfferResponse),
        (status = 400, description = "Error on McDonald's side"),
        (status = 409, description = "This deal is temporarily unavailable"),
        (status = 404, description = "No deals found of this type"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "deals",
)]
#[post("/deals/<offer_id>?<store>")]
pub async fn add_deal(
    ctx: &State<routes::Context>,
    offer_repository: &State<OfferRepository>,
    audit_repo: &State<AuditRepository>,
    account_repo: &State<AccountRepository>,
    offer_id: Either<i64, uuid::Uuid>,
    store: String,
    auth: RequiredAuthorizationHeader,
) -> Result<Json<OfferResponse>, ApiError> {
    let mut rng = StdRng::from_entropy();
    let locked_deals = offer_repository.get_locked_offers().await?;

    // "".parse::<Either<i32, uuid::Uuid>>();

    let deal_id = match offer_id {
        Either::Left(proposition_id) => {
            let all_deals: Vec<String> = offer_repository
                .get_offers_ids(&proposition_id.to_string())
                .await?
                .into_iter()
                .filter(|offer| !locked_deals.contains(offer))
                .collect();

            log::info!("found {} matching deals", all_deals.len());

            all_deals
                .choose(&mut rng)
                .context("must find at least one")
                .map_err(|_| ApiError::NotFound)?
                .to_owned()
        }
        Either::Right(deal_id) => deal_id.as_hyphenated().to_string(),
    };

    // need to catch errors and ensure the deal is unlocked to allow retries on 599
    let func = async {
        let (account, offer) = offer_repository.get_offer(&deal_id).await?;
        // lock the deal from appearing in GET /deals
        offer_repository
            .lock_offer(&deal_id, Duration::hours(DEFAULT_LOCK_TTL_HOURS))
            .await?;

        let proxy = proxy::get_proxy(&ctx.config.proxy).await;
        let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
        let api_client = account_repo
            .get_api_client(
                http_client,
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

        let resp = api_client
            .add_to_offers_dealstack(&offer_proposition_id, mc_donalds::default::OFFSET, &store)
            .await;

        if let Err(ref e) = resp {
            if let Some(status) = e.status() {
                // if adding to the deal stack fails, we fail...
                // we let the code above lock the deal though.
                // likely case is someone redeeming a deal but also removing it..
                // this lock will keep it removed and provide an error
                // 409 Conflict means the offer already exists
                // 404 when offer is already redeemed
                if !status.is_success() && status.as_u16() != 409 {
                    return Err(ApiError::McDonaldsError);
                }
            } else {
                return Err(e.into());
            }
        };

        // this can cause the offer id to change.. for offers with id == 0
        // we need to update the database to avoid inconsistency
        // but we also need to find the uuid for the refreshed deal and lock it
        // only need to do this when the deal was successfully added, otherwise the refresh won't
        // get any new offer_id
        if resp.is_ok() && offer_id == 0 {
            // consider doing this async...
            // may not be quick enough...
            log::info!("offer_id = 0, refreshing account: {}", account);
            let mut new_offers = offer_repository
                .refresh_offer_cache(
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

                    offer_repository.set_offers(&account, &new_offers).await?;
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

        // don't add to audit if it was a 409...
        let user_id = if resp.is_ok() {
            let user_name = &auth.claims.username;
            let user_id = &auth.claims.oid;

            audit_repo
                .add_entry(
                    AuditActionType::Add,
                    Some(user_id.to_string()),
                    user_name.to_string(),
                    &offer,
                )
                .await;

            Some(user_id.to_owned())
        } else {
            None
        };

        // if we get here with an error, it must be 409 because we return on other errors
        let resp = if let Ok(resp) = resp {
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
        } else {
            api_client
                .get_offers_dealstack(mc_donalds::default::OFFSET, &store)
                .await?
        };

        Ok(Json(OfferResponse {
            random_code: resp
                .body
                .response
                .expect("must have deal stack response")
                .random_code,
            message: resp
                .body
                .status
                .message
                .unwrap_or_else(|| "No message".to_string()),
            deal_uuid: Some(deal_id.to_string()),
        }))
    };

    match func.await {
        Ok(response) => Ok(response),
        Err(err) => {
            if let Err(e) = offer_repository.unlock_offer(&deal_id).await {
                log::error!("error unlocking deal after initial error: {}", e);
            }

            Err(err)
        }
    }
}
