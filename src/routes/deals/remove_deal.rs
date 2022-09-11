use crate::{client, constants::mc_donalds, routes, types::error::ApiError};
use rocket::{http::Status, State};

#[utoipa::path(
    responses(
        (status = 204, description = "Removed a deal"),
        (status = 400, description = "Error on McDonald's side"),
        (status = 404, description = "Deal not found"),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "deals",
)]
#[delete("/deals/<deal_id>?<store>")]
pub async fn remove_deal(
    ctx: &State<routes::Context<'_>>,
    deal_id: &str,
    store: i64,
) -> Result<Status, ApiError> {
    if let Ok((account, offer)) = ctx.database.get_offer_by_id(deal_id).await {
        let http_client = client::get_http_client();
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
        let resp = api_client
            .remove_from_offers_dealstack(
                &offer_id,
                &offer_proposition_id,
                mc_donalds::default::OFFSET,
                &store,
            )
            .await?;

        if resp.status.is_success() {
            ctx.database.unlock_deal(deal_id).await?;
            Ok(Status::NoContent)
        } else {
            Err(ApiError::McDonaldsError)
        }
    } else {
        Err(ApiError::NotFound)
    }
}
