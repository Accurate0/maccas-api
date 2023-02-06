use anyhow::Result;
use foundation::aws;
use foundation::http::get_default_http_client;
use lambda_http::service_fn;
use lambda_http::Error;
use lambda_runtime::LambdaEvent;
use libmaccas::types::request::ActivationRequest;
use libmaccas::types::request::Credentials;
use libmaccas::ApiClient;
use maccas::constants;
use maccas::database::{Database, DynamoDatabase};
use maccas::logging;
use maccas::types::config::GeneralConfig;
use maccas::types::sqs::SqsEvent;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rand::prelude::StdRng;
use rand::SeedableRng;
use regex::Regex;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;
extern crate imap;

#[tokio::main]
async fn main() -> Result<(), Error> {
    foundation::log::init_logger();
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(event: LambdaEvent<SqsEvent>) -> Result<(), anyhow::Error> {
    let now = Instant::now();
    let shared_config = aws::config::get_shared_config().await;
    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    if !config.accounts.enabled {
        log::warn!("accounts task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }
    let http_client = get_default_http_client();
    let mut client = ApiClient::new(
        constants::mc_donalds::default::BASE_URL.to_string(),
        http_client,
        config.mcdonalds.client_id,
    );

    let token_response = client
        .security_auth_token(&config.mcdonalds.client_secret)
        .await?;
    client.set_login_token(&token_response.body.response.token);

    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let imap_client = imap::connect(
        (config.accounts.email.server_address.clone(), 993),
        config.accounts.email.server_address,
        &tls,
    )?;
    let mut imap_session = imap_client
        .login(
            config.accounts.email.address,
            config.accounts.email.password,
        )
        .map_err(|e| e.0)?;

    let x = imap_session.select("INBOX")?.exists;

    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let db = DynamoDatabase::new(
        dynamodb_client,
        &config.database.tables,
        &config.database.indexes,
    );

    let messages = imap_session.fetch(format!("{}:{}", x, x - 20), "RFC822")?;
    log::info!("{}", messages.len());
    for message in messages.iter() {
        let body = message.body().expect("message did not have a body!");
        let body = std::str::from_utf8(body)
            .expect("message was not valid utf-8")
            .to_string();

        let re = Regex::new(r"ac=([A-Z0-9]+)").unwrap();
        let mut ac = None;
        for cap in re.captures_iter(&body) {
            ac = cap.get(1);
        }

        let re = Regex::new(r"To: ([a-zA-Z0-9]+)").unwrap();
        let mut to = None;
        for cap in re.captures_iter(&body) {
            to = cap.get(1);
        }

        if ac.is_some() && to.is_some() {
            let id = db
                .get_device_id_for(
                    format!("{}@{}", to.unwrap().as_str(), config.accounts.domain_name).as_str(),
                )
                .await?;
            log::info!("{:#?}", id);

            let device_id = id.unwrap_or_else(|| {
                let mut rng = StdRng::from_entropy();
                Alphanumeric.sample_string(&mut rng, 16)
            });
            log::info!("{:?}", ac.unwrap().as_str());
            log::info!("{:?}", to.unwrap().as_str().to_string());
            let request = ActivationRequest {
                activation_code: ac.unwrap().as_str().to_string()[2..ac.unwrap().as_str().len()]
                    .to_string(),
                credentials: Credentials {
                    login_username: format!(
                        "{}@{}",
                        to.unwrap().as_str(),
                        config.accounts.domain_name
                    ),
                    type_field: "device".to_string(),
                    password: None,
                },
                device_id: device_id.to_string(),
            };
            let response = client
                .put_customer_activation(&request, &config.mcdonalds.sensor_data)
                .await;

            log::info!("{:#?}", response);
            db.set_device_id_for(
                format!("{}@{}", to.unwrap().as_str(), config.accounts.domain_name).as_str(),
                device_id.as_str(),
            )
            .await
            .ok();
        }

        sleep(Duration::from_secs(5)).await;
    }

    imap_session.logout()?;
    log::info!(
        "completed accounts task in {} seconds",
        now.elapsed().as_secs()
    );
    Ok(())
}
