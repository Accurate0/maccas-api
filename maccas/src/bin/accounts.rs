use anyhow::Context;
use anyhow::Result;
use foundation::aws;
use foundation::http::get_default_http_client_with_proxy;
use lambda_http::service_fn;
use lambda_http::Error as LambdaError;
use lambda_runtime::LambdaEvent;
use libmaccas::types::request::ActivationRequest;
use libmaccas::types::request::Credentials;
use libmaccas::ApiClient;
use maccas::constants;
use maccas::database::{Database, DynamoDatabase};
use maccas::logging;
use maccas::proxy;
use maccas::types::config::GeneralConfig;
use maccas::types::sqs::SqsEvent;
use mailparse::MailHeaderMap;
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rand::prelude::StdRng;
use rand::SeedableRng;
use regex::Regex;
use std::error::Error;
use std::time::Duration;
use std::time::Instant;
use tokio::time::sleep;
extern crate imap;

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    foundation::log::init_logger(log::LevelFilter::Info, &[]);
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(event: LambdaEvent<SqsEvent>) -> Result<(), anyhow::Error> {
    let now = Instant::now();
    let shared_config = aws::config::get_shared_config().await;
    let config = GeneralConfig::load(&shared_config).await?;
    if !config.accounts.enabled {
        log::warn!("accounts task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }

    let proxy = proxy::get_proxy(&config.proxy).await;
    let http_client = get_default_http_client_with_proxy(proxy);
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

    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let db = DynamoDatabase::new(
        dynamodb_client,
        &config.database.tables,
        &config.database.indexes,
    );

    let re = Regex::new(r"ac=([A-Z0-9]+)").unwrap();
    let mut rng = StdRng::from_entropy();

    // get all
    imap_session.select("INBOX")?;
    let all_unseen_emails = imap_session.uid_search("(UNSEEN) SINCE 12-Oct-2023")?;
    log::info!("unseen messages: {}", all_unseen_emails.len());
    for message_uid in all_unseen_emails.iter() {
        let messages = imap_session.uid_fetch(message_uid.to_string(), "RFC822")?;
        let message = messages.first().context("should have at least one")?;
        let parsed_email = mailparse::parse_mail(message.body().context("must have body")?)?;

        let body = parsed_email.get_body()?;

        let mut ac = None;
        for cap in re.captures_iter(&body) {
            ac = cap.get(1);
        }

        let headers = parsed_email.get_headers();
        let to = headers.get_first_header("To");
        log::info!("email from: {:#?}", headers.get_first_header("Date"));

        log::info!("ac: {:?}, to: {:?}", ac, to);
        if ac.is_some() && to.is_some() {
            let to = to.unwrap().get_value();
            let id = db.get_device_id_for(&to).await?;
            log::info!("existing device id: {:?}", id);

            let device_id = id.unwrap_or_else(|| Alphanumeric.sample_string(&mut rng, 16));

            let ac = ac.unwrap().as_str();
            log::info!("code: {:?}", ac);
            log::info!("email to: {:?}", to.to_string());
            let request = ActivationRequest {
                activation_code: ac.to_string(),
                credentials: Credentials {
                    login_username: to.to_owned(),
                    type_field: "device".to_string(),
                    password: None,
                    send_magic_link: None,
                },
                device_id: device_id.to_string(),
            };
            match client
                .put_customer_activation(&request, &config.mcdonalds.sensor_data)
                .await
            {
                Ok(r) => log::info!(
                    "response: {} ({}) {:#?}",
                    r.status,
                    r.body.status.code,
                    r.body
                ),
                Err(e) => {
                    log::error!("status: {:?}, {:?}", e.status(), e.source());
                }
            };

            db.set_device_id_for(&to, &device_id).await.ok();
        }

        // there is a rate limit : )
        sleep(Duration::from_secs(5)).await;
    }

    imap_session.logout()?;
    log::info!(
        "completed accounts task in {} seconds",
        now.elapsed().as_secs()
    );
    Ok(())
}
