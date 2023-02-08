use crate::{
    constants::{config::MAX_PROXY_COUNT, mc_donalds},
    database::types::AuditActionType,
    guards::authorization::AuthorizationHeader,
    proxy,
    rng::RNG,
    routes,
    types::error::ApiError,
};
use foundation::types::jwt::JwtClaim;
use jwt::{Header, Token};
use rand::Rng;
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
    store: String,
    auth: AuthorizationHeader,
) -> Result<Status, ApiError> {
    if let Ok((account, offer)) = ctx.database.get_offer_by_id(deal_id).await {
        let mut rng = RNG.lock().await;
        let random_number = rng.gen_range(1..=MAX_PROXY_COUNT);

        let proxy = proxy::get_proxy(&ctx.config, random_number);
        let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
        let api_client = ctx
            .database
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
            let mut user_name: Option<String> = None;
            let mut user_id: Option<String> = None;
            if let Some(auth_header) = auth.0 {
                let auth_header = auth_header.replace("Bearer ", "");
                let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&auth_header)?;
                user_name = Some(jwt.claims().name.clone());
                user_id = Some(jwt.claims().oid.clone());
            }

            ctx.database
                .add_to_audit(AuditActionType::Remove, user_id, user_name, &offer)
                .await;
            ctx.database.unlock_deal(deal_id).await?;
            Ok(Status::NoContent)
        } else {
            Err(ApiError::McDonaldsError)
        }
    } else {
        Err(ApiError::NotFound)
    }
}
