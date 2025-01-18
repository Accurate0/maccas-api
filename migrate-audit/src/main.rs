use anyhow::Context;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::FixedOffset;
use entity::offer_details;
use sea_orm::{
    sea_query::OnConflict, ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait,
    QueryFilter, QuerySelect, Set,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr, time::Duration};
use tracing::{log::LevelFilter, Instrument, Level};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyOfferDatabase {
    pub deal_uuid: String,

    pub offer_id: i64,
    pub offer_proposition_id: i64,
    pub local_valid_from: String,
    pub local_valid_to: String,
    #[serde(rename = "validFromUTC")]
    pub valid_from_utc: String,
    #[serde(rename = "validToUTC")]
    pub valid_to_utc: String,
    pub name: String,
    pub short_name: String,
    pub description: String,
    #[serde(rename = "CreationDateUtc")]
    pub creation_date_utc: String,
    pub image_base_name: String,
    pub original_image_base_name: String,

    pub price: Option<f64>,
}

async fn insert_audit(
    input: &HashMap<String, AttributeValue>,
    db: &DatabaseConnection,
) -> anyhow::Result<()> {
    let offer_name = input
        .get("offer_name")
        .context("must have deal_uuid")?
        .as_s()
        .map_err(|_| anyhow::Error::msg("must be a string"))?;

    let existing_proposition_id = entity::offer_details::Entity::find()
        .filter(entity::offer_details::Column::ShortName.eq(offer_name))
        .limit(1)
        .one(db)
        .await?
        .map(|m| m.proposition_id);

    let proposition_id = if let Some(existing_proposition_id) = existing_proposition_id {
        existing_proposition_id
    } else {
        tracing::warn!("adding legacy details: {}", offer_name);

        let offer = input.get("offer").context("must have offer")?.clone();
        let legacy_offer = serde_dynamo::from_attribute_value::<_, LegacyOfferDatabase>(offer)?;

        let converted_offer = entity::offer_details::ActiveModel {
            proposition_id: Set(legacy_offer.offer_proposition_id),
            name: Set(legacy_offer.name),
            description: Set(legacy_offer.description),
            price: Set(legacy_offer.price),
            short_name: Set(legacy_offer.short_name),
            image_base_name: Set(legacy_offer.image_base_name),
            raw_data: Set(None),
            ..Default::default()
        };

        offer_details::Entity::insert(converted_offer)
            .on_conflict_do_nothing()
            .exec_without_returning(db)
            .await?;

        legacy_offer.offer_proposition_id
    };

    let id = input
        .get("deal_uuid")
        .context("must have id")?
        .as_s()
        .map_err(|_| anyhow::Error::msg("must be a string"))?;

    let action = if input
        .get("action")
        .context("must have action")?
        .as_s()
        .map_err(|_| anyhow::Error::msg("must be a string"))?
        == "Add"
    {
        entity::sea_orm_active_enums::Action::Add
    } else {
        entity::sea_orm_active_enums::Action::Remove
    };

    let user_id = input
        .get("user_id")
        .context("must have user_id")?
        .as_s()
        .map_err(|_| anyhow::Error::msg("must be a string"))?;

    let timestamp = chrono::DateTime::<FixedOffset>::parse_from_rfc3339(
        input
            .get("timestamp")
            .context("must have timestamp")?
            .as_s()
            .map_err(|_| anyhow::Error::msg("must be a string"))?,
    )?
    .naive_utc();

    let new_audit = entity::offer_audit::ActiveModel {
        action: Set(action),
        transaction_id: Set(uuid::Uuid::from_str(id)?),
        proposition_id: Set(proposition_id),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
        user_id: Set(uuid::Uuid::from_str(user_id).ok()),
        migrated: Set(true),
        ..Default::default()
    };

    tracing::info!("inserting: {}", offer_name);
    entity::offer_audit::Entity::insert(new_audit)
        .on_conflict(
            OnConflict::columns(vec![
                entity::offer_audit::Column::TransactionId,
                entity::offer_audit::Column::CreatedAt,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec_without_returning(db)
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    base::tracing::init("migrate-audit");
    let database_url = std::env::var("DATABASE_URL")?;

    let mut opt = ConnectOptions::new(database_url);
    opt.max_connections(1)
        .min_connections(1)
        .connect_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(LevelFilter::Trace);

    let db = Database::connect(opt).await?;

    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    async move {
        let mut scan_output = dynamodb_client
            .scan()
            .table_name("MaccasApi-Audit")
            .send()
            .await?;

        for item in scan_output.items() {
            insert_audit(item, &db)
                .instrument(tracing::span!(
                    Level::INFO,
                    "insert_audit",
                    "otel.name" = "insert_audit"
                ))
                .await?;
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
                    .table_name("MaccasApi-Audit")
                    .send()
                    .await?;
            }

            for item in scan_output.items() {
                insert_audit(item, &db)
                    .instrument(tracing::span!(
                        Level::INFO,
                        "insert_audit",
                        "otel.name" = "insert_audit"
                    ))
                    .await?;
            }
        }

        Ok::<(), anyhow::Error>(())
    }
    .instrument(tracing::span!(
        Level::INFO,
        "migrate",
        "otel.name" = "migrate"
    ))
    .await?;

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
