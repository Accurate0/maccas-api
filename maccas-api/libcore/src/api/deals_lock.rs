use super::Context;
use crate::lock;
use async_trait::async_trait;
use chrono::Duration;
use http::{Method, Response};
use lambda_http::{Body, Error, Request, RequestExt};
use simple_dispatcher::Executor;

pub struct DealsLock;

#[async_trait]
impl Executor<Context, Request, Response<Body>> for DealsLock {
    async fn execute(&self, ctx: &Context, request: &Request) -> Result<Response<Body>, Error> {
        let query_params = request.query_string_parameters();
        let deals = match request.body() {
            lambda_http::Body::Text(s) => match serde_json::from_str::<Vec<String>>(s) {
                Ok(obj) => obj,
                Err(_) => return Ok(Response::builder().status(400).body("".into())?),
            },
            _ => return Ok(Response::builder().status(400).body("".into())?),
        };

        Ok(match *request.method() {
            Method::POST => {
                let duration = query_params.first("duration").expect("must have duration");
                for deal_id in deals {
                    lock::lock_deal(
                        &ctx.dynamodb_client,
                        &ctx.api_config.offer_id_table_name,
                        &deal_id,
                        Duration::seconds(duration.parse::<i64>()?),
                    )
                    .await?;
                }
                Response::builder().status(204).body("".into())?
            }
            Method::DELETE => {
                for deal_id in deals {
                    lock::unlock_deal(&ctx.dynamodb_client, &ctx.api_config.offer_id_table_name, &deal_id).await?;
                }
                Response::builder().status(204).body("".into())?
            }
            _ => Response::builder().status(405).body("".into())?,
        })
    }
}
