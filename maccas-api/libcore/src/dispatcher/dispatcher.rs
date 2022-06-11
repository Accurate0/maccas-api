use crate::config::ApiConfig;
use http::{Request, Response};
use lambda_http::{request::RequestContext, Body, Error, RequestExt};
use std::collections::HashMap;

use super::Executor;

pub struct Dispatcher<'a> {
    config: &'a ApiConfig,
    dynamodb_client: &'a aws_sdk_dynamodb::Client,

    routes: HashMap<String, Box<&'a (dyn Executor + Send + Sync)>>,
    middleware: Vec<Box<&'a (dyn Executor + Send + Sync)>>,
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

    pub fn add_route(&mut self, path: &str, executor: &'a (dyn Executor + Send + Sync)) -> &mut Self {
        self.routes.insert(path.to_string(), Box::new(executor));
        self
    }

    pub fn add_middleware(&mut self, executor: &'a (dyn Executor + Send + Sync)) -> &mut Self {
        self.middleware.push(Box::new(executor));
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
