use crate::{constants::api_base, doc::openapi::ApiDoc, routes::Context};
use async_trait::async_trait;
use http::Response;
use lambda_http::{Body, IntoResponse, Request};
use simple_dispatcher::{Executor, ExecutorResult};
use utoipa::{
    openapi::{InfoBuilder, Server},
    OpenApi,
};

pub struct GetOpenApi;

#[async_trait]
impl Executor<Context<'_>, Request, Response<Body>> for GetOpenApi {
    async fn execute(&self, _ctx: &Context, _request: &Request) -> ExecutorResult<Response<Body>> {
        let mut spec = ApiDoc::openapi();
        let info = InfoBuilder::new().title("Maccas API").version("v1");
        spec.servers = Some(vec![Server::new(api_base::THIS)]);
        spec.info = info.build();

        Ok(spec.to_json()?.into_response())
    }
}
