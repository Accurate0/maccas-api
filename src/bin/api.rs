use anyhow::Context;
use http::{Response, StatusCode};
use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, Request, RequestExt};
use libapi::config::ApiConfig;
use libapi::database::DynamoDatabase;
use libapi::extensions::{RequestExtensions, ResponseExtensions};
use libapi::logging;
use libapi::routes;
use libapi::routes::fallback::Fallback;
use libapi::routes::user;
use libapi::routes::{code, statistics};
use libapi::routes::{deal, deals};
use libapi::routes::{locations, points};
use libapi::{constants, types};
use simple_dispatcher::RouteDispatcher;

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::setup_logging();
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;

    let config = ApiConfig::load_from_s3(&shared_config).await?;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let database = DynamoDatabase::new(&dynamodb_client, &config.tables);

    let context = routes::Context {
        config,
        database: Box::new(database),
    };

    let dispatcher = &RouteDispatcher::new(context, Fallback)
        .add_route("/deals", deals::Deals)
        .add_route("/points", points::Points)
        .add_route("/user/config", user::Config)
        .add_route("/deal/{dealId}", deal::Deal)
        .add_route("/code/{dealId}", code::Code)
        .add_route("/deals/lock", deals::LockUnlock)
        .add_route("/locations", locations::Locations)
        .add_route("/deals/{dealId}", deals::AddRemove)
        .add_route("/points/{accountId}", points::GetById)
        .add_route("/locations/search", locations::Search)
        .add_route("/deals/last-refresh", deals::LastRefresh)
        .add_route("/statistics/account", statistics::Account)
        .add_route("/statistics/total-accounts", statistics::TotalAccounts);

    let handler = move |request: Request| async move {
        request.log();

        let response = match dispatcher
            .dispatch(&request, || -> Option<String> {
                let context = request.request_context();
                match context {
                    RequestContext::ApiGatewayV1(r) => r.resource_path,
                    _ => None,
                }
            })
            .await
        {
            Ok(r) => r,
            Err(e) => {
                log::error!("{:?}", e);
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                Response::builder().status(status_code.as_u16()).body(
                    serde_json::to_string(&types::api::Error {
                        message: status_code.canonical_reason().context("no value")?.to_string(),
                    })?
                    .into(),
                )?
            }
        };

        response.log();
        Ok(response)
    };

    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}
