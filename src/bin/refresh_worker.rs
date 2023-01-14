use foundation::aws;
use maccas::{
    database::{types::UserAccountDatabase, Database, DynamoDatabase},
    proxy,
    types::{
        config::{GeneralConfig, UserList},
        sqs::RefreshFailureMessage,
    },
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    foundation::log::init_logger();
    let shared_config = aws::config::get_shared_config().await;
    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    let sqs_client = aws_sdk_sqs::Client::new(&shared_config);
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let database = DynamoDatabase::new(
        dynamodb_client,
        &config.database.tables,
        &config.database.indexes,
    );

    let total_users = UserList::load_all_from_s3(&shared_config).await?;
    log::info!("loaded {} accounts", total_users.users.len());

    loop {
        for user in total_users.users.iter().cycle() {
            let proxy = proxy::get_proxy(&config);
            let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
            let api_client_result = database
                .get_specific_client(
                    http_client,
                    &config.mcdonalds.client_id,
                    &config.mcdonalds.client_secret,
                    &config.mcdonalds.sensor_data,
                    &UserAccountDatabase::from(user),
                    false,
                )
                .await;

            match api_client_result {
                Ok(api_client) => {
                    database
                        .refresh_offer_cache_for(
                            &UserAccountDatabase::from(user),
                            &api_client,
                            &config.mcdonalds.ignored_offer_ids,
                        )
                        .await?;
                    database
                        .refresh_point_cache_for(&UserAccountDatabase::from(user), &api_client)
                        .await?;

                    // todo: add more handling, including email etc
                }
                Err(_) => {
                    foundation::aws::sqs::send_to_queue(
                        &sqs_client,
                        &config.refresh.failure_queue_name,
                        RefreshFailureMessage(user.clone()),
                    )
                    .await?;
                }
            };

            tokio::time::sleep(Duration::from_secs(120)).await;
        }
    }
}
