use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use libcore::api::Deals;
use libcore::api::DealsLock;
use libcore::api::LastRefresh;
use libcore::api::Locations;
use libcore::api::LocationsSearch;
use libcore::api::UserConfig;
use libcore::routes::Route;
use libcore::{config, constants, logging};

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

    let config = config::load_from_s3(&shared_config).await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let context = request.request_context();

    log::info!("request: {:#?}", request);

    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => return Ok(Response::builder().status(403).body("".into()).unwrap()),
    };

    let response = match resource_path {
        Some(s) => match s.as_str() {
            "/deals/last-refresh" => {
                LastRefresh::execute(&request, &dynamodb_client, &config).await?
            }

            "/locations" => Locations::execute(&request, &dynamodb_client, &config).await?,

            "/user/config" => UserConfig::execute(&request, &dynamodb_client, &config).await?,

            "/locations/search" => {
                LocationsSearch::execute(&request, &dynamodb_client, &config).await?
            }

            "/deals" => Deals::execute(&request, &dynamodb_client, &config).await?,

            "/deals/lock" => DealsLock::execute(&request, &dynamodb_client, &config).await?,

            _ => Response::builder().status(404).body("".into()).unwrap(),
        },
        None => Response::builder().status(400).body("".into()).unwrap(),
    };

    log::info!("response: {:#?}", response);
    Ok(response)
}
