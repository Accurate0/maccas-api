use lambda_http::Error as LambdaError;
use libapi::constants;
use libapi::database::DynamoDatabase;
use libapi::logging;
use libapi::routes;
use libapi::routes::admin::get_locked_deals::get_locked_deals;
use libapi::routes::admin::lock_deal::lock_deal;
use libapi::routes::admin::unlock_deal::unlock_deal;
use libapi::routes::catchers::{internal_server_error, not_found};
use libapi::routes::code::get_code::get_code;
use libapi::routes::deals::add_deal::add_deal;
use libapi::routes::deals::get_deal::get_deal;
use libapi::routes::deals::get_deals::get_deals;
use libapi::routes::deals::last_refresh::last_refresh;
use libapi::routes::deals::remove_deal::remove_deal;
use libapi::routes::docs::openapi::get_openapi;
use libapi::routes::locations::get_locations::get_locations;
use libapi::routes::locations::search::search_locations;
use libapi::routes::points::get_by_id::get_points_by_id;
use libapi::routes::points::get_points::get_points;
use libapi::routes::statistics::account::get_accounts;
use libapi::routes::statistics::total_accounts::get_total_accounts;
use libapi::routes::user::config::get_user_config;
use libapi::routes::user::config::update_user_config;
use libapi::types::config::GeneralConfig;
use rocket::http::Method;
use rocket::Config;
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use rocket_lamb::RocketExt;

#[macro_use]
extern crate rocket;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    logging::setup_logging();
    logging::dump_build_details();
    let is_aws = std::env::var(constants::AWS_LAMBDA_FUNCTION_NAME).is_ok();
    let shared_config = aws_config::from_env()
        .region(constants::DEFAULT_AWS_REGION)
        .load()
        .await;

    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    let sqs_client = aws_sdk_sqs::Client::new(&shared_config);
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let database = DynamoDatabase::new(dynamodb_client, &config.database.tables);

    let context = routes::Context {
        sqs_client,
        config,
        database: Box::new(database),
    };

    let config = Config {
        cli_colors: !is_aws,
        ..Default::default()
    };

    let rocket = rocket::build()
        .manage(context)
        .register("/", catchers![not_found, internal_server_error])
        .mount(
            "/",
            routes![
                get_code,
                get_openapi,
                get_deal,
                get_deals,
                add_deal,
                remove_deal,
                last_refresh,
                get_locations,
                search_locations,
                get_points_by_id,
                get_points,
                get_accounts,
                get_total_accounts,
                get_user_config,
                update_user_config,
                get_locked_deals,
                lock_deal,
                unlock_deal,
            ],
        )
        .configure(config);

    if is_aws {
        rocket.lambda().launch().await
    } else {
        let allowed_origins =
            AllowedOrigins::some_exact(&["https://maccas.anurag.sh", "http://localhost:3000"]);
        let cors = rocket_cors::CorsOptions {
            allowed_origins,
            allowed_methods: vec![Method::Get, Method::Post, Method::Delete]
                .into_iter()
                .map(From::from)
                .collect(),
            allowed_headers: AllowedHeaders::some(&[
                "Authorization",
                "Accept",
                "Content-Type",
                "X-Api-Key",
            ]),
            allow_credentials: true,
            ..Default::default()
        }
        .to_cors()?;
        match rocket.attach(cors).launch().await {
            Ok(_) => {
                log::info!("exiting...")
            }
            Err(e) => log::error!("error during launch: {}", e),
        };
    }

    Ok(())
}
