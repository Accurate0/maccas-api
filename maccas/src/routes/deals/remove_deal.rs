use crate::{
    constants::mc_donalds,
    database::{
        account::AccountRepository, audit::AuditRepository, offer::OfferRepository,
        types::AuditActionType,
    },
    guards::required_authorization::RequiredAuthorizationHeader,
    proxy, routes,
    types::error::ApiError,
};
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
    ctx: &State<routes::Context>,
    offer_repo: &State<OfferRepository>,
    audit_repo: &State<AuditRepository>,
    account_repo: &State<AccountRepository>,
    deal_id: &str,
    store: String,
    auth: RequiredAuthorizationHeader,
) -> Result<Status, ApiError> {
    if let Ok((account, offer)) = offer_repo.get_offer_by_id(deal_id).await {
        let proxy = proxy::get_proxy(&ctx.config.proxy).await;
        let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
        let api_client = account_repo
            .get_specific_client(
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
        let resp = api_client
            .remove_from_offers_dealstack(
                &offer_id,
                &offer_proposition_id,
                mc_donalds::default::OFFSET,
                &store,
            )
            .await?;

        if resp.status.is_success() {
            let user_id = auth.claims.oid;

            audit_repo
                .add_to_audit(
                    AuditActionType::Remove,
                    Some(user_id),
                    auth.claims.username,
                    &offer,
                )
                .await;
            offer_repo.unlock_deal(deal_id).await?;
            Ok(Status::NoContent)
        } else {
            Err(ApiError::McDonaldsError)
        }
    } else {
        Err(ApiError::NotFound)
    }
}
