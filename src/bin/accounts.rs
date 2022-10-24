use anyhow::Context;
use aws_sdk_dynamodb::Client;
use lambda_runtime::service_fn;
use lambda_runtime::{Error, LambdaEvent};
use libmaccas::types::request::{ActivationRequest, Credentials};
use libmaccas::ApiClient;
use maccas::constants;
use maccas::database::{Database, DynamoDatabase};
use maccas::logging;
use maccas::types::config::GeneralConfig;
use maccas::types::sqs::{FixAccountMessage, SqsEvent};
use maccas::{aws, client};
use rand::distributions::{Alphanumeric, DistString};
use rand::rngs::StdRng;
use rand::SeedableRng;
use regex::{Regex, RegexBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
    logging::setup_logging();
    logging::dump_build_details();
    lambda_runtime::run(service_fn(run)).await?;
    Ok(())
}

async fn run(event: LambdaEvent<SqsEvent>) -> Result<(), anyhow::Error> {
    let shared_config = aws::get_shared_config().await;

    let config = GeneralConfig::load_from_s3(&shared_config).await?;
    if !config.accounts.enabled {
        log::warn!("accounts task is disabled, ignoring event: {:?}", &event);
        return Ok(());
    }

    let client = Client::new(&shared_config);
    let database: Box<dyn Database> =
        Box::new(DynamoDatabase::new(client, &config.database.tables));

    let mut valid_records = event.payload.records;
    valid_records.retain(|msg| msg.body.is_some());

    let messages: Vec<FixAccountMessage> = valid_records
        .iter()
        .map(|msg| {
            serde_json::from_str(msg.body.as_ref().unwrap())
                .context("must deserialize")
                .unwrap()
        })
        .collect();

    let mut rng = StdRng::from_entropy();
    let http_client = client::get_http_client();
    let mut api_client = ApiClient::new(
        constants::mc_donalds::default::BASE_URL.to_string(),
        &http_client,
        config.mcdonalds.client_id.clone(),
    );

    let token_response = api_client
        .security_auth_token(&config.mcdonalds.client_secret)
        .await?;
    api_client.set_login_token(&token_response.body.response.token);

    let tls = native_tls::TlsConnector::builder().build().unwrap();
    let addr = config.accounts.imap_address.to_string();
    let imap_client = imap::connect((addr.clone(), config.accounts.imap_port), addr, &tls)?;
    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = imap_client
        .login(config.accounts.email, config.accounts.password)
        .map_err(|e| e.0)?;
    // we want to fetch the first email in the INBOX mailbox
    let x = imap_session.select("INBOX")?.exists;
    let emails = imap_session.fetch(
        format!("{}:{}", x, x - config.accounts.check_last_email_count),
        "RFC822",
    )?;

    let activation_code_regex = Regex::new(r"ac=([A-Z0-9]+)").unwrap();
    let to_regex = RegexBuilder::new(r"^To: <?([a-zA-Z0-9@.]+)>?$")
        .multi_line(true)
        .build()
        .unwrap();

    // batch size is currently 1 so this loop is redundant..
    for message in messages {
        log::info!("request: {:?}", message);
        let login_email = &message.account.login_username;
        // need to find the latest, must run this in reverse
        for email in emails.iter().rev() {
            // extract the message's body
            let body = email.body().expect("email did not have a body!");
            let body = std::str::from_utf8(body)
                .expect("email was not valid utf-8")
                .to_string();

            let body_without_crlf = body.clone().replace("\r\n", "\n");

            let to = to_regex
                .captures_iter(&body_without_crlf)
                .next()
                .context("invalid email, no To");

            if to.is_err() {
                continue;
            }

            let to = to.unwrap().get(1);

            if to.is_none() {
                continue;
            }

            let to = to.unwrap().as_str();

            log::info!("checking email to: {} vs {}", to, login_email);
            if to == login_email {
                log::info!("match found for {}", login_email);
                let activation_code = activation_code_regex
                    .captures_iter(&body)
                    .next()
                    .context("invalid email, no activation code")?
                    .get(1)
                    .context("invalid email, no activation code")?
                    .as_str();

                // ensure this is device verification
                body.contains("CLICK HERE TO VERIFY YOUR DEVICE")
                    .then_some(0)
                    .context("must be device verification")?;

                // encoded '=' at the start as 3D
                let activation_code =
                    activation_code.to_string()[2..activation_code.len()].to_string();

                let id = database.get_device_id_for(to).await?;
                log::info!("device id: {:?}", id);

                let device_id = id.unwrap_or_else(|| Alphanumeric.sample_string(&mut rng, 16));
                log::info!("activation code: {:?}", activation_code);
                log::info!("email: {:?}", to.to_string());
                let request = ActivationRequest {
                    activation_code,
                    credentials: Credentials {
                        login_username: to.to_string(),
                        type_field: "device".to_string(),
                        password: None,
                    },
                    device_id: device_id.to_string(),
                };
                let response = api_client
                    .put_customer_activation(&request, &config.mcdonalds.sensor_data)
                    .await?;

                log::info!("response: {:#?}", response);
                database.set_device_id_for(to, device_id.as_str()).await?;

                log::info!("found latest email for this account: {}", login_email);
                log::info!("finishing search");

                log::info!("refreshing cache for {}", login_email);

                let api_client = database
                    .get_specific_client(
                        &http_client,
                        &config.mcdonalds.client_id,
                        &config.mcdonalds.client_secret,
                        &config.mcdonalds.sensor_data,
                        &message.account,
                        false,
                    )
                    .await?;

                database
                    .refresh_offer_cache_for(
                        &message.account,
                        &api_client,
                        &config.mcdonalds.ignored_offer_ids,
                    )
                    .await?;

                // just the first one, it's the latest
                break;
            }
        }
    }

    imap_session.logout()?;
    Ok(())
}
