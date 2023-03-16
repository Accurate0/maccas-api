use crate::{
    constants::config::CONFIG_APIM_API_KEY_ID,
    guards::admin::AdminOnlyRoute,
    routes,
    shared::spending::generate_spending_information,
    types::{
        api::{AdminUserSpending, AdminUserSpendingMap},
        error::ApiError,
    },
};
use foundation::{
    constants::{CORRELATION_ID_HEADER, GRAPH_API_BASE_URL, X_API_KEY_HEADER},
    extensions::SecretsManagerExtensions,
    rocket::guards::correlation_id::CorrelationId,
    types::graph::GetUsersResponse,
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
    params(
        ("X-Maccas-JWT-Bypass" = Option<String>, Header, description = "Key to bypass JWT checks"),
    ),
)]
#[get("/admin/user/spending")]
pub async fn get_all_user_spending(
    ctx: &State<routes::Context<'_>>,
    correlation_id: CorrelationId,
    _admin: AdminOnlyRoute,
) -> Result<Json<AdminUserSpendingMap>, ApiError> {
    let http_client = foundation::http::get_default_http_client();
    let user_list = http_client
        .get(format!("{GRAPH_API_BASE_URL}/users"))
        .header(
            X_API_KEY_HEADER,
            &ctx.secrets_client
                .get_secret(CONFIG_APIM_API_KEY_ID)
                .await?,
        )
        .header(CORRELATION_ID_HEADER, correlation_id.0)
        .send()
        .await?
        .json::<GetUsersResponse>()
        .await?
        .value;

    let spending_futures = user_list
        .iter()
        .map(|u| ctx.database.get_audit_entries_for(u.id.to_owned()))
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
                .display_name
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
