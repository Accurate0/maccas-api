use crate::routes::Context;
use async_trait::async_trait;
use chrono::Duration;
use http::{Method, Response};
use lambda_http::{Body, Request, RequestExt};
use simple_dispatcher::{Executor, ExecutorResult};

pub struct LockUnlock;

pub mod docs {
    #[utoipa::path(
        post,
        path = "/deals/lock",
        request_body(
            content = [String],
            content_type = "application/json",
        ),
        responses(
            (status = 204, description = "Locked deals"),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        params(
            ("duration" = i64, path, description = "the lock duration in seconds"),
        ),
        tag = "deals",
    )]
    #[deprecated]
    pub fn lock_deal() {}

    #[utoipa::path(
        delete,
        path = "/deals/lock",
        request_body(
            content = [String],
            content_type = "application/json",
        ),
        responses(
            (status = 204, description = "Unlocked deals"),
            (status = 500, description = "Internal Server Error", body = Error),
        ),
        tag = "deals",
    )]
    #[deprecated]
    pub fn unlock_deal() {}
}

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for LockUnlock {
    async fn execute(&self, ctx: &Context, request: &Request) -> ExecutorResult<Response<Body>> {
        let query_params = request.query_string_parameters();
        let deals = match request.body() {
            lambda_http::Body::Text(s) => match serde_json::from_str::<Vec<String>>(s) {
                Ok(obj) => obj,
                Err(_) => return Ok(Response::builder().status(400).body(Body::Empty)?),
            },
            _ => return Ok(Response::builder().status(400).body(Body::Empty)?),
        };

        Ok(match *request.method() {
            Method::POST => {
                let duration = query_params.first("duration").expect("must have duration");
                for deal_id in deals {
                    ctx.database
                        .lock_deal(&deal_id, Duration::seconds(duration.parse::<i64>()?))
                        .await?;
                }
                Response::builder().status(204).body(Body::Empty)?
            }
            Method::DELETE => {
                for deal_id in deals {
                    ctx.database.unlock_deal(&deal_id).await?;
                }
                Response::builder().status(204).body(Body::Empty)?
            }
            _ => Response::builder().status(405).body(Body::Empty)?,
        })
    }
}
