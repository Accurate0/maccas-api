use anyhow::Context;
use clap::{Parser, Subcommand};
use foundation::aws;
use libmaccas::{
    types::request::{
        AcceptancePolicies, ActivateAndSignInRequest, ActivationDevice, ActivationRequest, Address,
        Audit, ClientInfo, Credentials, Device, Policies, Preference, RegistrationRequest,
        Subscription,
    },
    ApiClient,
};
use maccas::{
    constants,
    database::{Database, DynamoDatabase},
    types::config::GeneralConfig,
};
use mailparse::MailHeaderMap;
use rand::{
    distributions::{Alphanumeric, DistString},
    rngs::StdRng,
    SeedableRng,
};
use regex::Regex;
use std::{error::Error, time::Duration};
use titlecase::titlecase;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "false")]
    dry_run: bool,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    CreateAccounts {
        #[arg(short, long)]
        count: u32,
        #[arg(short, long)]
        group: i64,
        #[arg(short, long)]
        region: String,
    },
    ActivateAccounts {
        #[arg(short, long)]
        count: u32,
    },
    ActivateAndLogin,
    ActivateAccount {
        #[arg(short, long)]
        email: String,
        #[arg(short, long)]
        activation_code: String,
    },
}

// TODO: upgrade cli to use mailparse too
// TODO: use unseen lookup, smarter...
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    foundation::log::init_logger(log::LevelFilter::Info, &[]);
    maccas::logging::dump_build_details();

    let shared_config = aws::config::get_shared_config().await;
    let args = Args::parse();
    let real_run = !args.dry_run;

    log::info!("dry run enabled: {}", args.dry_run);

    let config = GeneralConfig::load(&shared_config).await?;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
    let database = DynamoDatabase::new(
        dynamodb_client,
        &config.database.tables,
        &config.database.indexes,
    );

    let proxy = maccas::proxy::get_proxy(&config.proxy).await;
    let http_client = foundation::http::get_default_http_client_with_proxy(proxy);
    let mut client = ApiClient::new(
        constants::mc_donalds::default::BASE_URL.to_string(),
        http_client,
        config.mcdonalds.client_id,
    );

    let response = client
        .security_auth_token(&config.mcdonalds.client_secret)
        .await?;
    client.set_login_token(&response.body.response.token);

    let mut rng = StdRng::from_entropy();

    match args.command {
        Commands::CreateAccounts {
            count,
            group,
            region,
        } => {
            for _ in 0..count {
                let firstname = petname::Petnames::default().generate(&mut rng, 1, "");
                let lastname = petname::Petnames::default().generate(&mut rng, 1, "");

                let device_id = Alphanumeric.sample_string(&mut rng, 16);
                let username =
                    format!("{}.{}@{}", firstname, lastname, config.accounts.domain_name);

                let request = RegistrationRequest {
                    address: Address {
                        country: "AU".to_string(),
                        zip_code: "6233".to_string(),
                    },
                    audit: Audit {
                        registration_channel: "M".to_string(),
                    },
                    credentials: Credentials {
                        login_username: username.to_string(),
                        password: None,
                        send_magic_link: Some(true),
                        type_field: "email".to_string(),
                    },
                    device: Device {
                        device_id: device_id.to_string(),
                        device_id_type: "AndroidId".to_string(),
                        is_active: "Y".to_string(),
                        os: "android".to_string(),
                        os_version: "13".to_string(),
                        timezone: "Australia/West".to_string(),
                    },
                    email_address: username.to_string(),
                    first_name: titlecase(&firstname),
                    last_name: titlecase(&lastname),
                    opt_in_for_marketing: false,
                    policies: Policies {
                        acceptance_policies: AcceptancePolicies { n1: true, n4: true },
                    },
                    preferences: serde_json::from_str::<Vec<Preference>>(include_str!(
                        "../files/preferences.json"
                    ))
                    .unwrap(),
                    subscriptions: serde_json::from_str::<Vec<Subscription>>(include_str!(
                        "../files/subscriptions.json"
                    ))
                    .unwrap(),
                };

                if real_run {
                    client
                        .customer_registration(&request, &config.mcdonalds.sensor_data)
                        .await?;
                }

                log::info!(
                    "[{}] created account with name {} {}",
                    request.email_address,
                    firstname,
                    lastname
                );

                if real_run {
                    database
                        .add_user_account(
                            &username,
                            &username,
                            "(UNUSED)",
                            &region,
                            &group.to_string(),
                        )
                        .await?;

                    database.set_device_id_for(&username, &device_id).await?;
                }

                log::info!("[{}] added to database", request.email_address);
                if real_run {
                    log::info!("sleeping for 10 seconds");
                    tokio::time::sleep(Duration::from_secs(10)).await
                } else {
                    log::info!("dry run, not created");
                }
            }
        }
        Commands::ActivateAccounts { count } => {
            log::info!("attempting to activate accounts");
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

            log::info!("total messages: {}", x);
            let messages = imap_session.fetch(format!("{}:{}", x, x - count + 1), "RFC822")?;
            log::info!("messages: {}", messages.len());
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

                let re = Regex::new(r"To: ([a-zA-Z0-9\.]+)").unwrap();
                let mut to = None;
                for cap in re.captures_iter(&body) {
                    to = cap.get(1);
                }

                if ac.is_some() && to.is_some() {
                    let id = db
                        .get_device_id_for(
                            format!("{}@{}", to.unwrap().as_str(), config.accounts.domain_name)
                                .as_str(),
                        )
                        .await?;
                    log::info!("existing device id: {:?}", id);

                    let device_id = id.unwrap_or_else(|| {
                        let mut rng = StdRng::from_entropy();
                        Alphanumeric.sample_string(&mut rng, 16)
                    });
                    let ac = &ac.unwrap().as_str().to_string()[2..ac.unwrap().as_str().len()];
                    log::info!("code: {:?}", ac);
                    log::info!("email to: {:?}", to.unwrap().as_str().to_string());
                    let request = ActivationRequest {
                        activation_code: ac.to_string(),
                        credentials: Credentials {
                            login_username: format!(
                                "{}@{}",
                                to.unwrap().as_str(),
                                config.accounts.domain_name
                            ),
                            type_field: "email".to_string(),
                            password: None,
                            send_magic_link: None,
                        },
                        device_id: device_id.to_string(),
                    };

                    if real_run {
                        let response = client
                            .put_customer_activation(&request, &config.mcdonalds.sensor_data)
                            .await;
                        log::info!("{:?}", response);
                    } else {
                        log::info!("dry run, not activating");
                    }

                    db.set_device_id_for(
                        format!("{}@{}", to.unwrap().as_str(), config.accounts.domain_name)
                            .as_str(),
                        device_id.as_str(),
                    )
                    .await?;

                    if real_run {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
            }
        }
        Commands::ActivateAndLogin => {
            // TODO: actually finish this, not needed yet apparently.. things kinda just work..
            log::info!("attempting to activate and login accounts");
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

            imap_session.select("INBOX")?;
            let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
            let db = DynamoDatabase::new(
                dynamodb_client,
                &config.database.tables,
                &config.database.indexes,
            );

            let all_unseen_emails = imap_session.uid_search("(UNSEEN)")?;
            for message_uid in all_unseen_emails.iter() {
                let messages = imap_session.uid_fetch(message_uid.to_string(), "RFC822")?;
                let message = messages.first().context("should have at least one")?;
                let body = message.body().expect("message did not have a body!");

                let parsed_email = mailparse::parse_mail(body)?;
                let body: &String = &parsed_email.get_body()?;
                let re = Regex::new(r"ml=([a-zA-Z0-9]+)").unwrap();
                let mut magic_link = None;
                for cap in re.captures_iter(body) {
                    log::info!("capture: {:?}", cap);
                    magic_link = cap.get(1);
                }

                let headers = parsed_email.get_headers();
                let to = headers
                    .get_first_header("To")
                    .context("must have to")?
                    .get_value();
                let from = headers
                    .get_first_header("From")
                    .context("must have from")?
                    .get_value();

                if !from.contains("accounts@au.mcdonalds.com") {
                    log::warn!("skipping non maccas email, {:?}", from);
                    continue;
                }

                if magic_link.is_some() {
                    let id = db.get_device_id_for(&to).await?;
                    log::info!("existing device id: {:?}", id);

                    let device_id = id.unwrap_or_else(|| {
                        let mut rng = StdRng::from_entropy();
                        Alphanumeric.sample_string(&mut rng, 16)
                    });
                    // let ac = &magic_link.unwrap().as_str().to_string()
                    //     [2..magic_link.unwrap().as_str().len()];
                    let magic_link = magic_link.unwrap().as_str().to_string();
                    log::info!("code: {:?}", magic_link);
                    log::info!("email to: {:?}", to.as_str().to_string());

                    if real_run {
                        let response = client
                            .activate_and_signin(
                                &ActivateAndSignInRequest {
                                    activation_link: magic_link,
                                    client_info: ClientInfo {
                                        device: ActivationDevice {
                                            device_unique_id: device_id.to_owned(),
                                            os: "android".to_owned(),
                                            os_version: "13".to_owned(),
                                        },
                                    },
                                },
                                &config.mcdonalds.sensor_data,
                            )
                            .await?;
                        log::info!("{:?}", response);
                        db.set_device_id_for(&to, device_id.as_str()).await?;
                        if let Some(token_response) = response.body.response {
                            db.set_access_and_refresh_token_for(
                                &to,
                                &token_response.access_token,
                                &token_response.refresh_token,
                            )
                            .await?;
                        }
                    } else {
                        log::info!("dry run, not activating");
                    }
                }

                if real_run {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
        Commands::ActivateAccount {
            email,
            activation_code,
        } => {
            let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);
            let db = DynamoDatabase::new(
                dynamodb_client,
                &config.database.tables,
                &config.database.indexes,
            );
            let device_id = db.get_device_id_for(&email).await?;
            log::info!("existing device id: {:?}", device_id);

            let device_id = device_id.unwrap_or_else(|| Alphanumeric.sample_string(&mut rng, 16));

            log::info!("code: {:?}", activation_code);
            log::info!("email to: {:?}", email);
            let request = ActivationRequest {
                activation_code,
                credentials: Credentials {
                    login_username: email.to_owned(),
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

            db.set_device_id_for(&email, &device_id).await.ok();
        }
    }

    Ok(())
}
