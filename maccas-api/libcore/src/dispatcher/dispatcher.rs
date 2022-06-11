use std::collections::HashMap;
use http::{Request, Response};
use lambda_http::{Body, Error, RequestExt, request::RequestContext};
use crate::config::ApiConfig;

use super::Executor;

pub struct Dispatcher<'a> {
    mappings: HashMap<String, Box<&'a (dyn Executor + Send + Sync)>>,
    middleware: Vec<Box<&'a (dyn Executor + Send + Sync)>>,
}

impl<'a> Dispatcher<'a> {
    pub fn new() -> Self {
        Self {
            mappings:  HashMap::new(),
            middleware: Vec::new()
        }
    }

    pub fn add_route(&mut self, path: &str, route: &'a (dyn Executor + Send + Sync)) {
        self.mappings.insert(path.to_string(), Box::new(route));
    }

    pub fn add_middleware(&mut self, middleware: &'a (dyn Executor + Send + Sync)) {
        self.middleware.push(Box::new(middleware));
    }

    pub async fn execute(
        &self,
        request: &Request<Body>,
        dynamodb_client: &aws_sdk_dynamodb::Client,
        config: &ApiConfig,
    ) -> Result<Response<Body>, Error> {
        let context = request.request_context();

        let resource_path = match context {
            RequestContext::ApiGatewayV1(r) => r.resource_path,
            _ => return Ok(Response::builder().status(403).body("".into())?),
        };

        if let Some(resource_path) = resource_path {
            if let Some(route) = self.mappings.get(&resource_path) {
                for middleware in &self.middleware {
                    middleware.execute(&request, &dynamodb_client, &config).await?;
                }

                Ok(route.execute(&request, &dynamodb_client, &config).await?)
            } else {
                Ok(Response::builder().status(404).body("".into())?)
            }
        }
        else {
            Ok(Response::builder().status(404).body("".into())?)
        }
    }
}
