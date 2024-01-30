use std::collections::HashMap;

use aws_config::{retry::RetryConfig, BehaviorVersion};
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use entity::accounts::{self};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, Set};
use uuid::Uuid;

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
        count: usize,
    },
}

async fn get_and_insert_new(
    item: &HashMap<String, AttributeValue>,
    dynamodb_client: &aws_sdk_dynamodb::Client,
    db: &DatabaseConnection,
) -> Result<(), anyhow::Error> {
    tracing::info!("copying info: {:?}", item);
    let account_name = item.get("account_name").unwrap().as_s().unwrap().to_owned();
    let resp = dynamodb_client
        .get_item()
        .table_name("MaccasApi-Tokens")
        .key("account_name", AttributeValue::S(account_name.to_owned()))
        .send()
        .await?;
    let details = resp.item().unwrap();
    let password = item
        .get("login_password")
        .unwrap()
        .as_s()
        .unwrap()
        .to_owned();

    let account_model = accounts::ActiveModel {
        id: Set(Uuid::new_v4()),
        username: Set(item
            .get("login_username")
            .unwrap()
            .as_s()
            .unwrap()
            .to_owned()),
        password: Set(if password == "(UNUSED)" {
            None
        } else {
            Some(password)
        }),
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

        device_id: Set(details.get("device_id").unwrap().as_s().unwrap().to_owned()),
        updated_at: Set(NaiveDateTime::UNIX_EPOCH),
        ..Default::default()
    };

    account_model.insert(db).await?;

    dynamodb_client
        .delete_item()
        .table_name("MaccasApi-Tokens")
        .key("account_name", AttributeValue::S(account_name.to_owned()))
        .send()
        .await?;

    dynamodb_client
        .delete_item()
        .table_name("MaccasApi-UserAccounts")
        .key("account_name", AttributeValue::S(account_name.to_owned()))
        .send()
        .await?;

    dynamodb_client
        .delete_item()
        .table_name("MaccasApi-Accounts")
        .key("account_name", AttributeValue::S(account_name.to_owned()))
        .send()
        .await?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    tracing_subscriber::fmt().init();
    let db = Database::connect(args.database_url).await?;
    let mut current_count = 0;

    let shared_config = aws_config::defaults(BehaviorVersion::latest())
        .region("ap-southeast-2")
        .retry_config(RetryConfig::standard())
        .load()
        .await;

    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);

    match args.command {
        Commands::CopyAccounts { count: max_count } => {
            let mut scan_output = dynamodb_client
                .scan()
                .table_name("MaccasApi-UserAccounts".to_owned())
                .send()
                .await?;

            for item in scan_output.items() {
                match get_and_insert_new(item, &dynamodb_client, &db).await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("{}", e),
                }

                current_count += 1;
                if current_count >= max_count {
                    return Ok(());
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
                    match get_and_insert_new(item, &dynamodb_client, &db).await {
                        Ok(_) => {}
                        Err(e) => tracing::error!("{}", e),
                    }

                    current_count += 1;
                    if current_count > max_count {
                        return Ok(());
                    }
                }
            }
        }
    }

    Ok(())
}
