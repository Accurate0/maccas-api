use crate::constants::mc_donalds;
use crate::guards::authorization::AuthorizationHeader;
use crate::guards::correlation_id::CorrelationId;
use crate::logging::log_deal_use;
use crate::types::api::OfferResponse;
use crate::types::error::ApiError;
use crate::{client, routes};
use anyhow::Context;
use chrono::Duration;
use rocket::http::Status;
use rocket::{serde::json::Json, State};

pub struct AddRemove;

#[utoipa::path(
        post,
        path = "/deals/{dealId}",
        responses(
            (status = 200, description = "Added a deal", body = OfferResponse),
            (status = 400, description = "Error on McDonald's side", body = ApiError),
            (status = 404, description = "Deal not found", body = ApiError),
            (status = 500, description = "Internal Server Error", body = ApiError),
        ),
        params(
            ("dealId" = String, path, description = "The deal id to add"),
            ("store" = Option<i64>, query, description = "The selected store"),
        ),
        tag = "deals",
    )]
#[post("/deals/<deal_id>?<store>")]
pub async fn add_deal(
    ctx: &State<routes::Context<'_>>,
    deal_id: &str,
    store: Option<i64>,
    auth: AuthorizationHeader,
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
        let short_name = offer.short_name.to_string();

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

                    ctx.database.set_offers_for(&account.account_name, &new_offers).await?;
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
            if auth.0.is_some() {
                log_deal_use(
                    &http_client,
                    auth.0.unwrap().as_str(),
                    &correlation_id.0,
                    &short_name,
                    deal_id,
                    &offer.image_base_name,
                    &ctx.config,
                )
                .await;
            }
            resp
        };

        let resp = OfferResponse::from(resp.body);
        Ok(Json(resp))
    } else {
        Err(ApiError::NotFound)
    }
}

#[utoipa::path(
        delete,
        path = "/deals/{dealId}",
        responses(
            (status = 204, description = "Removed a deal"),
            (status = 400, description = "Error on McDonald's side"),
            (status = 404, description = "Deal not found"),
            (status = 500, description = "Internal Server Error"),
        ),
        params(
            ("dealId" = String, path, description = "The deal id to remove"),
        ),
        tag = "deals",
    )]
#[delete("/deals/<deal_id>?<store>")]
pub async fn remove_deal(
    ctx: &State<routes::Context<'_>>,
    deal_id: &str,
    store: Option<i64>,
) -> Result<Status, ApiError> {
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
        let resp = api_client
            .remove_from_offers_dealstack(
                &offer_id,
                &offer_proposition_id,
                mc_donalds::default::OFFSET,
                &store.unwrap_or(mc_donalds::default::STORE_ID),
            )
            .await?;

        if resp.status.is_success() {
            ctx.database.unlock_deal(deal_id).await?;
            Ok(Status::NoContent)
        } else {
            Err(ApiError::NotFound)
        }
    } else {
        Err(ApiError::NotFound)
    }
}
