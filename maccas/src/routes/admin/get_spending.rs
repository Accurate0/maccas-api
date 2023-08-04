use crate::{
    constants::{config::CONFIG_AD_CLIENT_SECRET_KEY_ID, graph},
    guards::admin::AdminOnlyRoute,
    routes,
    shared::spending::generate_spending_information,
    types::{
        api::{AdminUserSpending, AdminUserSpendingMap},
        error::ApiError,
    },
};
use foundation::{extensions::SecretsManagerExtensions, types::graph::GetUsersResponse};
use graph_rs_sdk::{oauth::AccessToken, Graph};
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
    let token_request_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        ctx.config.api.graph.tenant_id
    );

    let mut oauth_client = graph_rs_sdk::oauth::OAuth::new();
    let oauth_client = oauth_client
        .client_id(&ctx.config.api.jwt.application_id)
        .access_token_url(&token_request_url)
        .client_secret(
            &ctx.secrets_client
                .get_secret(CONFIG_AD_CLIENT_SECRET_KEY_ID)
                .await?,
        )
        .add_scope(graph::DEFAULT_CLIENT_SCOPE)
        .build_async();

    let token_response = oauth_client
        .client_credentials()
        .access_token()
        .send()
        .await?
        .json::<AccessToken>()
        .await?;

    let mut graph_client = Graph::new(token_response.bearer_token());
    let graph_client = graph_client.beta();

    let user_list = graph_client
        .users()
        .list_user()
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
