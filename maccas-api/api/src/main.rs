use lambda_http::{service_fn, Error, IntoResponse, Request};
use libcore::api::DealsAddRemove;
use libcore::api::{Code, Deals, DealsLock, LastRefresh, Locations, LocationsSearch, UserConfig};
use libcore::config::ApiConfig;
use libcore::constants;
use libcore::dispatcher::Dispatcher;
use libcore::extensions::{RequestExtensions, ResponseExtensions};
use libcore::logging;

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::setup_logging();
    lambda_http::run(service_fn(run)).await?;
    Ok(())
}

async fn run(request: Request) -> Result<impl IntoResponse, Error> {
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;

    let config = ApiConfig::load_from_s3(&shared_config).await?;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    request.log();

    let mut dispatcher = Dispatcher::new();

    dispatcher.add_route("/deals", &Deals);
    dispatcher.add_route("/code/{dealId}", &Code);
    dispatcher.add_route("/locations", &Locations);
    dispatcher.add_route("/deals/lock", &DealsLock);
    dispatcher.add_route("/user/config", &UserConfig);
    dispatcher.add_route("/deals/{dealId}", &DealsAddRemove);
    dispatcher.add_route("/deals/last-refresh", &LastRefresh);
    dispatcher.add_route("/locations/search", &LocationsSearch);

    let response = dispatcher.execute(&request, &dynamodb_client, &config).await?;
    response.log();

    Ok(response)
}
