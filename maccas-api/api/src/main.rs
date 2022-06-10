use lambda_http::request::RequestContext;
use lambda_http::{service_fn, Error, IntoResponse, Request, RequestExt, Response};
use libcore::api::Code;
use libcore::api::DealsAddRemove;
use libcore::config;
use libcore::constants;
use libcore::logging;
use libcore::routes::Route;

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

    let resource_path = match context {
        RequestContext::ApiGatewayV1(r) => r.resource_path,
        _ => return Ok(Response::builder().status(403).body("".into()).unwrap()),
    };

    log::info!("request: {:#?}", request);

    let response = match resource_path {
        Some(path) => match path.as_str() {
            "/code/{dealId}" => Code::execute(&request, &dynamodb_client, &config).await?,

            "/deals/{dealId}" => {
                DealsAddRemove::execute(&request, &dynamodb_client, &config).await?
            }

            _ => Response::builder().status(400).body("".into()).unwrap(),
        },
        _ => Response::builder().status(400).body("".into()).unwrap(),
    };

    log::info!("response: {:#?}", response);
    Ok(response)
}
