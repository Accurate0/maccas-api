use crate::{config::ApiConfig, dispatcher::Executor, lock};
use async_trait::async_trait;
use chrono::Duration;
use http::{Method, Response};
use lambda_http::{Body, Error, Request, RequestExt};

pub struct DealsLock;

#[async_trait]
impl Executor for DealsLock {
    async fn execute(
        &self,
        request: &Request,
        dynamodb_client: &aws_sdk_dynamodb::Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
        let query_params = request.query_string_parameters();
        let deals = match request.body() {
            lambda_http::Body::Text(s) => match serde_json::from_str::<Vec<String>>(s) {
                Ok(obj) => obj,
                Err(_) => return Ok(Response::builder().status(400).body("".into()).unwrap()),
            },
            _ => return Ok(Response::builder().status(400).body("".into()).unwrap()),
        };

        Ok(match *request.method() {
            Method::POST => {
                let duration = query_params.first("duration").expect("must have duration");
                for deal_id in deals {
                    lock::lock_deal(
                        &dynamodb_client,
                        &config.offer_id_table_name,
                        &deal_id,
                        Duration::seconds(duration.parse::<i64>().unwrap()),
                    )
                    .await?;
                }
                Response::builder().status(204).body("".into()).unwrap()
            }
            Method::DELETE => {
                for deal_id in deals {
                    lock::unlock_deal(&dynamodb_client, &config.offer_id_table_name, &deal_id).await?;
                }
                Response::builder().status(204).body("".into()).unwrap()
            }
            _ => Response::builder().status(405).body("".into()).unwrap(),
        })
    }
}
