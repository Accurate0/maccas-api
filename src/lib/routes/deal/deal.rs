use crate::routes::Context;
use async_trait::async_trait;
use http::{
    header::{CACHE_CONTROL, CONTENT_TYPE},
    Response, StatusCode,
};
use lambda_http::{Body, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct Deal;

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
            Response::builder().status(404).body(Body::Empty)?
        })
    }
}
