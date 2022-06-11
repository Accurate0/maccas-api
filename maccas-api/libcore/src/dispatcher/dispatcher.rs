use super::Executor;
use crate::config::ApiConfig;
use http::{Request, Response};
use lambda_http::{request::RequestContext, Body, Error, RequestExt};
use std::{collections::HashMap, sync::Arc};

type DynamicExecutor = dyn Executor + Send + Sync;

pub struct Dispatcher<'a> {
    config: &'a ApiConfig,
    dynamodb_client: &'a aws_sdk_dynamodb::Client,

    fallback: Option<Arc<DynamicExecutor>>,
    routes: HashMap<String, Arc<DynamicExecutor>>,
}

impl<'a> Dispatcher<'a> {
    pub fn new(config: &'a ApiConfig, dynamodb_client: &'a aws_sdk_dynamodb::Client) -> Self {
        Self {
            routes: HashMap::new(),
            config,
            dynamodb_client,
            fallback: None,
        }
    }

    pub fn add_route<E: 'static>(mut self, path: &str, executor: E) -> Self
    where
        E: Executor + Send + Sync,
    {
        self.routes.insert(path.to_string(), Arc::new(executor));
        self
    }

    pub fn set_fallback<E: 'static>(mut self, executor: E) -> Self
    where
        E: Executor + Send + Sync,
    {
        self.fallback = Some(Arc::new(executor));
        self
    }

    async fn execute_fallback(&self, request: &Request<Body>) -> Result<Response<Body>, Error> {
        if let Some(fallback) = &self.fallback {
            Ok(fallback.execute(&request, &self.dynamodb_client, &self.config).await?)
        } else {
            Ok(Response::builder().status(404).body("".into())?)
        }
    }

    pub async fn dispatch(&self, request: &Request<Body>) -> Result<Response<Body>, Error> {
        let context = request.request_context();
        let resource_path = match context {
            RequestContext::ApiGatewayV1(r) => r.resource_path,
            _ => return Ok(Response::builder().status(403).body("".into())?),
        };

        if let Some(resource_path) = resource_path {
            if let Some(route) = self.routes.get(&resource_path) {
                Ok(route.execute(&request, &self.dynamodb_client, &self.config).await?)
            } else {
                Ok(self.execute_fallback(&request).await?)
            }
        } else {
            Ok(self.execute_fallback(&request).await?)
        }
    }
}
