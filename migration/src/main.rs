use anyhow::Context;
use clap::{Parser, Subcommand};
use foundation::aws;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    CopyDatabase {
        #[arg(short, long)]
        from: String,
        #[arg(short, long)]
        to: String,
    },
    UpdateSchema,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    foundation::log::init_logger(log::LevelFilter::Info, &[]);
    maccas::logging::dump_build_details();

    let shared_config = aws::config::get_shared_config().await;
    let args = Args::parse();
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&shared_config);

    match args.command {
        Commands::CopyDatabase { from, to } => {
            let mut scan_output = dynamodb_client
                .scan()
                .table_name(from.clone())
                .send()
                .await?;
            for item in scan_output.items().context("must have items")? {
                log::info!("copying item {:?} to {}", item, to);
                let mut put_item = dynamodb_client.put_item().table_name(to.clone());
                for (k, v) in item {
                    put_item = put_item.item(k, v.clone());
                }

                put_item.send().await?;
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
                        .table_name(from.clone())
                        .send()
                        .await?;
                }

                for item in scan_output.items().context("must have items")? {
                    log::info!("copying item {:?} to {}", item, to);
                    let mut put_item = dynamodb_client.put_item().table_name(to.clone());
                    for (k, v) in item {
                        put_item = put_item.item(k, v.clone());
                    }

                    put_item.send().await?;
                }
            }

            log::info!("copied table {from} to {to}");
        }
        Commands::UpdateSchema => todo!(),
    }

    Ok(())
}
