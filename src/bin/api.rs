use lambda_http::Error as LambdaError;
use maccas::constants;
use maccas::database::DynamoDatabase;
use maccas::logging;
use maccas::routes;
use maccas::routes::admin::get_locked_deals::get_locked_deals;
use maccas::routes::admin::lock_deal::lock_deal;
use maccas::routes::admin::unlock_deal::unlock_deal;
use maccas::routes::catchers::{internal_server_error, not_found};
use maccas::routes::code::get_code::get_code;
use maccas::routes::deals::add_deal::add_deal;
use maccas::routes::deals::get_deal::get_deal;
use maccas::routes::deals::get_deals::get_deals;
use maccas::routes::deals::get_last_refresh::last_refresh;
use maccas::routes::deals::remove_deal::remove_deal;
use maccas::routes::docs::openapi::get_openapi;
use maccas::routes::locations::get_locations::get_locations;
use maccas::routes::locations::search_locations::search_locations;
use maccas::routes::points::get_by_id::get_points_by_id;
use maccas::routes::points::get_points::get_points;
use maccas::routes::statistics::get_account::get_accounts;
use maccas::routes::statistics::get_total_accounts::get_total_accounts;
use maccas::routes::user::config::get_user_config;
use maccas::routes::user::config::update_user_config;
use maccas::types::config::GeneralConfig;
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
