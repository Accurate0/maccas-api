use crate::config::ApiConfig;
use http::{Request, Response};
use lambda_http::{request::RequestContext, Body, Error, RequestExt};
use std::{collections::HashMap, sync::Arc};

use super::Executor;

pub struct Dispatcher<'a> {
    config: &'a ApiConfig,
    dynamodb_client: &'a aws_sdk_dynamodb::Client,

    routes: HashMap<String, Arc<dyn Executor + Send + Sync>>,
    middleware: Vec<Arc<(dyn Executor + Send + Sync)>>,
}

impl<'a> Dispatcher<'a> {
    pub fn new(config: &'a ApiConfig, dynamodb_client: &'a aws_sdk_dynamodb::Client) -> Self {
        Self {
            routes: HashMap::new(),
            middleware: Vec::new(),
            config,
            dynamodb_client,
        }
    }

    pub fn add_route<E: 'static>(mut self, path: &str, executor: E) -> Self
    where
        E: Executor + Send + Sync,
    {
        self.routes.insert(path.to_string(), Arc::new(executor));
        self
    }

    pub fn add_middleware<E: 'static>(mut self, executor: E) -> Self
    where
        E: Executor + Send + Sync,
    {
        self.middleware.push(Arc::new(executor));
        self
    }

    pub async fn dispatch(&self, request: &Request<Body>) -> Result<Response<Body>, Error> {
        let context = request.request_context();

        let resource_path = match context {
            RequestContext::ApiGatewayV1(r) => r.resource_path,
            _ => return Ok(Response::builder().status(403).body("".into())?),
        };

        if let Some(resource_path) = resource_path {
            if let Some(route) = self.routes.get(&resource_path) {
                for middleware in &self.middleware {
                    middleware
                        .execute(&request, &self.dynamodb_client, &self.config)
                        .await?;
                }

                Ok(route.execute(&request, &self.dynamodb_client, &self.config).await?)
            } else {
                Ok(Response::builder().status(404).body("".into())?)
            }
        } else {
            Ok(Response::builder().status(404).body("".into())?)
        }
    }
}
