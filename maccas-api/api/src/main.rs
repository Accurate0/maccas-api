use lambda_http::{service_fn, Error, Request};
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
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;

    let config = ApiConfig::load_from_s3(&shared_config).await?;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);

    let mut dispatcher = Dispatcher::new(&config, &dynamodb_client);

    dispatcher.add_route("/deals", &Deals);
    dispatcher.add_route("/code/{dealId}", &Code);
    dispatcher.add_route("/locations", &Locations);
    dispatcher.add_route("/deals/lock", &DealsLock);
    dispatcher.add_route("/user/config", &UserConfig);
    dispatcher.add_route("/deals/{dealId}", &DealsAddRemove);
    dispatcher.add_route("/deals/last-refresh", &LastRefresh);
    dispatcher.add_route("/locations/search", &LocationsSearch);

    let dispatcher = &dispatcher;

    let handler_func_closure = move |request: Request| async move {
        request.log();
        let response = dispatcher.dispatch(&request).await?;
        response.log();
        Ok(response)
    };

   // Pass the closure to the runtime here.
   lambda_http::run(service_fn(handler_func_closure)).await?;

    Ok(())
}
