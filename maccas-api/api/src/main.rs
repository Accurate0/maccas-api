use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, Request, RequestExt};
use libcore::api::{Code, Context, Deals, DealsLock, LastRefresh, Locations, LocationsSearch, UserConfig};
use libcore::api::{DealsAddRemove, Fallback};
use libcore::config::ApiConfig;
use libcore::constants;
use libcore::extensions::RequestExtensions;
use libcore::logging;
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

    let context = Context {
        config,
        dynamodb_client,
    };

    let ref dispatcher = RouteDispatcher::new(context, Fallback)
        .add_route("/deals", Deals)
        .add_route("/code/{dealId}", Code)
        .add_route("/locations", Locations)
        .add_route("/deals/lock", DealsLock)
        .add_route("/user/config", UserConfig)
        .add_route("/deals/{dealId}", DealsAddRemove)
        .add_route("/deals/last-refresh", LastRefresh)
        .add_route("/locations/search", LocationsSearch);

    let handler = move |request: Request| async move {
        request.log();

        let response = dispatcher
            .dispatch(&request, || -> Option<String> {
                let context = request.request_context();
                match context {
                    RequestContext::ApiGatewayV1(r) => r.resource_path,
                    _ => None,
                }
            })
            .await?;

        Ok(response)
    };

    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}
