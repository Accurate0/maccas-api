use crate::{
    guards::admin::AdminOnlyRoute,
    routes,
    shared::spending::generate_spending_information,
    types::{
        api::{AdminUserSpending, AdminUserSpendingMap},
        error::ApiError,
    },
};
use itertools::Itertools;
use rocket::{futures::future::try_join_all, serde::json::Json, State};
use std::collections::HashMap;

#[utoipa::path(
    responses(
        (status = 200, description = "List of currently locked deals", body = AdminUserSpendingMap),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
)]
#[get("/admin/user/spending")]
pub async fn get_all_user_spending(
    ctx: &State<routes::Context<'_>>,
    _admin: AdminOnlyRoute,
) -> Result<Json<AdminUserSpendingMap>, ApiError> {
    let user_list = ctx.database.get_all_users().await?;

    let spending_futures = user_list
        .iter()
        .map(|u| ctx.database.get_audit_entries_for(u.id.to_string()))
        .collect_vec();

    let spending_map: HashMap<_, _> = try_join_all(spending_futures)
        .await?
        .into_iter()
        .filter(|e| !e.is_empty())
        .map(|e| {
            let spending_information = generate_spending_information(&e);
            let user_id = e.first().unwrap().user_id.clone();
            (spending_information, user_id)
        })
        .filter(|(spending_information, _)| !spending_information.items.is_empty())
        .map(|(spending_information, user_id)| {
            let name = user_list
                .iter()
                .find(|u| u.id == user_id)
                .unwrap()
                .username
                .to_string();
            (
                user_id,
                AdminUserSpending {
                    total: spending_information.total,
                    items: spending_information.items,
                    name,
                },
            )
        })
        .collect();

    Ok(Json(AdminUserSpendingMap(spending_map)))
}
