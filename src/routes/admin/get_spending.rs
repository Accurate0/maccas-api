use std::collections::HashMap;

use crate::{
    guards::admin::AdminOnlyRoute,
    routes,
    shared::generate_spending_information,
    types::{api::AdminUserSpending, error::ApiError},
};
use itertools::Itertools;
use rocket::{futures::future::try_join_all, serde::json::Json, State};

#[utoipa::path(
    responses(
        (status = 200, description = "List of currently locked deals", body = AdminUserSpending),
        (status = 500, description = "Internal Server Error"),
    ),
    tag = "admin",
    params(
        ("X-Maccas-JWT-Bypass" = Option<String>, Header, description = "Key to bypass JWT checks"),
    ),
)]
#[get("/admin/user/spending")]
pub async fn get_all_user_spending(
    ctx: &State<routes::Context<'_>>,
    _admin: AdminOnlyRoute,
) -> Result<Json<AdminUserSpending>, ApiError> {
    let user_id_list = ctx.database.get_all_user_ids_from_audit().await?;
    let spending_futures = user_id_list
        .iter()
        .map(|u| ctx.database.get_audit_entries_for(u))
        .collect_vec();

    let spending_map: HashMap<_, _> = try_join_all(spending_futures)
        .await?
        .into_iter()
        .map(|e| {
            (
                e.first().unwrap().user_id.clone(),
                generate_spending_information(&e),
            )
        })
        .collect();

    Ok(Json(AdminUserSpending(spending_map)))
}
