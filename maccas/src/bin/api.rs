use async_graphql::EmptyMutation;
use async_graphql::EmptySubscription;
use async_graphql::Schema;
use foundation::aws;
use foundation::constants::AWS_LAMBDA_FUNCTION_NAME;
use lambda_http::Error as LambdaError;
use maccas::database::account::AccountRepository;
use maccas::database::audit::AuditRepository;
use maccas::database::offer::OfferRepository;
use maccas::database::point::PointRepository;
use maccas::database::refresh::RefreshRepository;
use maccas::database::user::UserRepository;
use maccas::logging;
use maccas::routes;
use maccas::routes::admin::get_locked_deals::get_locked_deals;
use maccas::routes::admin::get_spending::get_all_user_spending;
use maccas::routes::admin::lock_deal::lock_deal;
use maccas::routes::admin::register::registration_token;
use maccas::routes::admin::unlock_deal::unlock_deal;
use maccas::routes::auth::login::login;
use maccas::routes::auth::register::register;
use maccas::routes::auth::token::get_token;
use maccas::routes::catchers::default;
use maccas::routes::code::get_code::get_code;
use maccas::routes::deals::add_deal::add_deal;
use maccas::routes::deals::get_deal::get_deal;
use maccas::routes::deals::get_deals::get_deals;
use maccas::routes::deals::get_last_refresh::get_last_refresh;
use maccas::routes::deals::remove_deal::remove_deal;
use maccas::routes::docs::graphql::get_graphql;
use maccas::routes::docs::openapi::get_openapi;
use maccas::routes::graphql::graphiql;
use maccas::routes::graphql::graphql_query;
use maccas::routes::graphql::graphql_request;
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
use maccas::routes::Database;
use maccas::types::config::GeneralConfig;
use rocket::config::Ident;
use rocket::Config;

#[cfg(debug_assertions)]
use {http::header::ORIGIN, rocket::fairing::AdHoc, rocket::http::Method, rocket::http::Status};

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<(), LambdaError> {
    foundation::log::init_logger();
    logging::dump_build_details();
    let is_aws = std::env::var(AWS_LAMBDA_FUNCTION_NAME).is_ok();
    let shared_config = aws::config::get_shared_config().await;

    let config = GeneralConfig::load(&shared_config).await?;
    let sqs_client = aws_sdk_sqs::Client::new(&shared_config);
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let secrets_client = aws_sdk_secretsmanager::Client::new(&shared_config);

    let user_repository = UserRepository::new(dynamodb_client.clone(), &config.database.tables);
    let offer_repository = OfferRepository::new(
        dynamodb_client.clone(),
        &config.database.tables,
        &config.database.indexes,
    );
    let audit_repository = AuditRepository::new(
        dynamodb_client.clone(),
        &config.database.tables,
        &config.database.indexes,
    );
    let account_repository =
        AccountRepository::new(dynamodb_client.clone(), &config.database.tables);
    let point_repository = PointRepository::new(dynamodb_client.clone(), &config.database.tables);
    let refresh_repository =
        RefreshRepository::new(dynamodb_client.clone(), &config.database.tables);

    let rocket = rocket::build();
    let context = routes::Context {
        secrets_client,
        sqs_client,
        config,
        database: Database {
            user_repository: user_repository.clone(),
            account_repository: account_repository.clone(),
            audit_repository: audit_repository.clone(),
            offer_repository: offer_repository.clone(),
            point_repository: point_repository.clone(),
            refresh_repository: refresh_repository.clone(),
        },
    };

    let config = Config {
        cli_colors: !is_aws,
        ident: Ident::none(),
        ..Default::default()
    };

    let schema = Schema::build(
        maccas::graphql::query::QueryRoot::default(),
        EmptyMutation,
        EmptySubscription,
    )
    .data(context.clone())
    .finish();

    let rocket = rocket
        .manage(context)
        .manage(user_repository)
        .manage(offer_repository)
        .manage(audit_repository)
        .manage(account_repository)
        .manage(point_repository)
        .manage(refresh_repository)
        .manage(schema)
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
                login,
                get_token,
                register,
                registration_token,
                graphiql,
                graphql_query,
                graphql_request,
                get_graphql
            ],
        )
        .configure(config);

    #[cfg(debug_assertions)]
    let rocket = {
        rocket.attach(AdHoc::on_response("cors", |req, response| {
            Box::pin(async move {
                let is_options = req.method() == Method::Options;
                let origin = req.headers().get_one(ORIGIN.as_str());
                let is_localhost = origin.map_or(false, |o| o.starts_with("http://localhost"));

                if is_localhost {
                    response.set_raw_header("Access-Control-Allow-Origin", origin.unwrap_or(""));
                }

                if is_options && is_localhost {
                    response.set_raw_header("Access-Control-Allow-Methods", "GET,POST,DELETE");
                    response.set_raw_header(
                        "Access-Control-Allow-Headers",
                        "Authorization,Content-Type",
                    );
                    response.set_raw_header("Access-Control-Allow-Credentials", "true");
                    response.set_raw_header("Access-Control-Max-Age", "259200");
                    response.set_status(Status::Ok)
                }
            })
        }))
    };

    match rocket.launch().await {
        Ok(_) => {
            log::info!("exiting...")
        }
        Err(e) => log::error!("error during launch: {}", e),
    };

    Ok(())
}
