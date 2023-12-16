use aws_config::{retry::RetryConfig, BehaviorVersion};
use aws_sdk_dynamodb::types::AttributeValue;
use clap::{Parser, Subcommand};
use entity::accounts::{self};
use sea_orm::{ActiveModelTrait, Database, Set};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    database_url: String,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    CopyAccounts {
        #[arg(short, long)]
        update: bool,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();
    let db = Database::connect(args.database_url).await?;

    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region("ap-southeast-2")
        .retry_config(RetryConfig::standard())
        .load()
        .await;

    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);

    match args.command {
        Commands::CopyAccounts { update } => {
            let mut scan_output = dynamodb_client
                .scan()
                .table_name("MaccasApi-UserAccounts".to_owned())
                .send()
                .await?;

            for item in scan_output.items() {
                tracing::info!("copying info: {:?}", item);
                let account_name = item.get("account_name").unwrap().as_s().unwrap().to_owned();
                let resp = dynamodb_client
                    .get_item()
                    .table_name("MaccasApi-Tokens")
                    .key("account_name", AttributeValue::S(account_name.to_owned()))
                    .send()
                    .await?;
                let details = resp.item().unwrap();

                let account_model = accounts::ActiveModel {
                    name: Set(account_name),
                    login_username: Set(item
                        .get("login_username")
                        .unwrap()
                        .as_s()
                        .unwrap()
                        .to_owned()),
                    login_password: Set(item
                        .get("login_password")
                        .unwrap()
                        .as_s()
                        .unwrap()
                        .to_owned()),
                    access_token: Set(details
                        .get("access_token")
                        .unwrap()
                        .as_s()
                        .unwrap()
                        .to_owned()),
                    refresh_token: Set(details
                        .get("refresh_token")
                        .unwrap()
                        .as_s()
                        .unwrap()
                        .to_owned()),

                    ..Default::default()
                };

                if update {
                    account_model.save(&db).await?;
                } else {
                    account_model.insert(&db).await?;
                }
            }

            // keep going until no more last evaluated key
            loop {
                let last_key = scan_output.last_evaluated_key();
                if scan_output.last_evaluated_key().is_none() {
                    break;
                } else {
                    scan_output = dynamodb_client
                        .scan()
                        .set_exclusive_start_key(last_key.cloned())
                        .table_name("MaccasApi-UserAccounts".to_owned())
                        .send()
                        .await?;
                }

                for item in scan_output.items() {
                    tracing::info!("copying info: {:?}", item);
                    let account_name = item.get("account_name").unwrap().as_s().unwrap().to_owned();
                    let resp = dynamodb_client
                        .get_item()
                        .table_name("MaccasApi-Tokens")
                        .key("account_name", AttributeValue::S(account_name.to_owned()))
                        .send()
                        .await?;
                    let details = resp.item().unwrap();

                    let account_model = accounts::ActiveModel {
                        name: Set(account_name),
                        login_username: Set(item
                            .get("login_username")
                            .unwrap()
                            .as_s()
                            .unwrap()
                            .to_owned()),
                        login_password: Set(item
                            .get("login_password")
                            .unwrap()
                            .as_s()
                            .unwrap()
                            .to_owned()),
                        access_token: Set(details
                            .get("access_token")
                            .unwrap()
                            .as_s()
                            .unwrap()
                            .to_owned()),
                        refresh_token: Set(details
                            .get("refresh_token")
                            .unwrap()
                            .as_s()
                            .unwrap()
                            .to_owned()),

                        ..Default::default()
                    };

                    if update {
                        account_model.save(&db).await?;
                    } else {
                        account_model.insert(&db).await?;
                    }
                }
            }
        }
    }

    Ok(())
}
