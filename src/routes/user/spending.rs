use crate::{
    database::types::AuditActionType,
    guards::authorization::RequiredAuthorizationHeader,
    routes,
    types::api::UserSpending,
    types::{api::GetDealsOffer, audit::AuditEntry, error::ApiError, jwt::JwtClaim},
};
use itertools::Itertools;
use jwt::{Header, Token};
use rocket::{serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "Spending information for this user", body = UserSpending),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "user",
)]
#[get("/user/spending")]
pub async fn get_user_spending(
    ctx: &State<routes::Context<'_>>,
    auth: RequiredAuthorizationHeader,
) -> Result<Json<UserSpending>, ApiError> {
    let value = auth.0.as_str().replace("Bearer ", "");
    let jwt: Token<Header, JwtClaim, _> = jwt::Token::parse_unverified(&value)?;
    let user_id = &jwt.claims().oid;

    let entries = ctx
        .database
        .get_audit_entries_for(user_id)
        .await
        .unwrap_or_default();

    let items = entries
        .iter()
        .sorted_by_key(|entry| entry.offer.deal_uuid.clone())
        .group_by(|entry| entry.offer.deal_uuid.clone())
        .into_iter()
        .map(|e| e.1.collect::<Vec<&AuditEntry>>())
        .collect_vec();

    let mut final_list = Vec::new();
    for item in &items {
        let mut list = Vec::new();
        let all_add_count = item
            .iter()
            .filter(|e| e.action == AuditActionType::Add)
            .collect_vec()
            .len();

        let all_remove_count = item
            .iter()
            .filter(|e| e.action == AuditActionType::Remove)
            .collect_vec()
            .len();

        let final_count = all_add_count - all_remove_count;

        if final_count > 0 {
            list.append(
                &mut item
                    .iter()
                    .take(final_count)
                    .map(|e| GetDealsOffer::from(e.offer.clone()))
                    .collect_vec(),
            );
        }

        final_list.append(&mut list);
    }

    let total_cost = entries
        .iter()
        .map(|entry| {
            let price = entry.offer.price.unwrap_or_default();
            match entry.action {
                AuditActionType::Add => price,
                AuditActionType::Remove => -price,
            }
        })
        .sum();

    Ok(Json(UserSpending {
        total: total_cost,
        items: final_list,
    }))
}
