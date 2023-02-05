use foundation::aws;
use foundation::constants::AWS_LAMBDA_FUNCTION_NAME;
use lambda_http::Error as LambdaError;
use maccas::database::DynamoDatabase;
use maccas::extensions::SecretsManagerExtensions;
use maccas::logging;
use maccas::routes;
use maccas::routes::admin::get_locked_deals::get_locked_deals;
use maccas::routes::admin::get_spending::get_all_user_spending;
use maccas::routes::admin::lock_deal::lock_deal;
use maccas::routes::admin::unlock_deal::unlock_deal;
use maccas::routes::catchers::default;
use maccas::routes::code::get_code::get_code;
use maccas::routes::deals::add_deal::add_deal;
use maccas::routes::deals::get_deal::get_deal;
use maccas::routes::deals::get_deals::get_deals;
use maccas::routes::deals::get_last_refresh::get_last_refresh;
use maccas::routes::deals::remove_deal::remove_deal;
use maccas::routes::docs::openapi::get_openapi;
use maccas::routes::health::status::get_status;
use maccas::routes::locations::get_locations::get_locations;
use maccas::routes::locations::search_locations::search_locations;
use maccas::routes::points::get_by_id::get_points_by_id;
use maccas::routes::points::get_points::get_points;
use maccas::routes::statistics::get_account::get_accounts;
use maccas::routes::statistics::get_total_accounts::get_total_accounts;
use maccas::routes::user::config::get_user_config;
use maccas::routes::user::config::update_user_config;
use maccas::routes::user::spending::get_user_spending;
use maccas::types::config::GeneralConfig;
use rocket::config::Ident;
use rocket::Config;
use std::time::Duration;

#[cfg(debug_assertions)]
use {
    rocket::http::Method,
    rocket_cors::{AllowedHeaders, AllowedOrigins},
};

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
    foundation::log::init_logger();
    logging::dump_build_details();
    let is_aws = std::env::var(AWS_LAMBDA_FUNCTION_NAME).is_ok();
    let shared_config = aws::config::get_shared_config().await;

    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    let sqs_client = aws_sdk_sqs::Client::new(&shared_config);
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let secrets_client = aws_sdk_secretsmanager::Client::new(&shared_config);
    let database = DynamoDatabase::new(
        dynamodb_client,
        &config.database.tables,
        &config.database.indexes,
    );

    let rocket = rocket::build();
    let rocket = if config.api.jwt.validate {
        let authority = foundation::jwt::create_authority_with_defaults(
            config.api.jwt.jwks_url.clone(),
            secrets_client.get_application_id().await,
        )
        .await?;
        // not sure if needed
        authority.spawn_refresh(Duration::from_secs(60));
        rocket.manage(authority)
    } else {
        rocket
    };

    let context = routes::Context {
        secrets_client,
        sqs_client,
        config,
        database: Box::new(database),
    };

    let config = Config {
        cli_colors: !is_aws,
        ident: Ident::none(),
        ..Default::default()
    };

    let rocket = rocket
        .manage(context)
        .register("/", catchers![default])
        .mount(
            "/",
            routes![
                get_status,
                get_code,
                get_openapi,
                get_deal,
                get_deals,
                add_deal,
                remove_deal,
                get_last_refresh,
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
                get_user_spending,
                get_all_user_spending,
            ],
        )
        .configure(config);

    #[cfg(debug_assertions)]
    let rocket = {
        let allowed_origins =
            AllowedOrigins::some_exact(&["https://dev.maccas.anurag.sh", "http://localhost:3000"]);
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
                "X-Maccas-JWT-Bypass",
            ]),
            allow_credentials: true,
            ..Default::default()
        }
        .to_cors()?;
        rocket.attach(cors)
    };

    match rocket.launch().await {
        Ok(_) => {
            log::info!("exiting...")
        }
        Err(e) => log::error!("error during launch: {}", e),
    };

    Ok(())
}
