use crate::{routes::Context, types::api::Error};
use async_trait::async_trait;
use http::{
    header::{CACHE_CONTROL, CONTENT_TYPE},
    Response, StatusCode,
};
use lambda_http::{Body, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Deal;

pub mod docs {
    #[utoipa::path(
        get,
        path = "/deal/{dealId}",
        responses(
            (status = 200, description = "Information for specified deal", body = Offer),
            (status = 404, description = "Deal not found", body = Error),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        params(
            ("dealId" = String, path, description = "The deal id to add"),
        ),
        tag = "deals",
    )]
    pub fn get_deal() {}
}

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for Deal {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let path_params = request.path_parameters();

        let deal_id = path_params.first("dealId").expect("must have id");
        let deal_id = &deal_id.to_owned();

        Ok(if let Ok((_, offer)) = ctx.database.get_offer_by_id(deal_id).await {
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, mime::APPLICATION_JSON.to_string())
                .header(CACHE_CONTROL, "max-age=900")
                .body(serde_json::to_string(&offer)?.into())?
        } else {
            let status_code = StatusCode::NOT_FOUND;
            Response::builder().status(status_code.as_u16()).body(
                serde_json::to_string(&Error {
                    message: status_code.canonical_reason().ok_or("no value")?.to_string(),
                })?
                .into(),
            )?
        })
    }
}
