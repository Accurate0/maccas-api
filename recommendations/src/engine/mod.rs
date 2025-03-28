use entity::{offer_details, offer_embeddings};
use error::RecommendationError;
use openai::types::OpenAIEmbeddingsRequest;
use sea_orm::prelude::PgVector;
use sea_orm::sea_query::{OnConflict, Query};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use std::{ops::Deref, sync::Arc};
use tracing::instrument;

mod error;

#[derive(Clone, Debug)]
pub struct RecommendationEngine {
    inner: Arc<EngineInner>,
}

#[derive(Debug)]
pub struct EngineInner {
    db: DatabaseConnection,
    openai_api_client: openai::ApiClient,
}

impl Deref for RecommendationEngine {
    type Target = EngineInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RecommendationEngine {
    pub fn new(db: DatabaseConnection, openai_api_client: openai::ApiClient) -> Self {
        Self {
            inner: Arc::new(EngineInner {
                db,
                openai_api_client,
            }),
        }
    }

    pub async fn is_healthy(&self) -> bool {
        self.db.ping().await.ok().is_some()
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    #[instrument(skip(self))]
    pub async fn generate_clusters(&self) {}

    #[instrument(skip(self))]
    pub async fn refresh_all_embeddings(&self) {
        let s = self.clone();
        tokio::spawn(async move {
            if let Err(e) = s.refresh_all_embeddings_internal().await {
                tracing::error!("{e}");
            }
        });
    }

    #[instrument(skip(self))]
    pub async fn refresh_embedding_for(
        &self,
        proposition_id: i64,
        input: String,
        force: bool,
    ) -> Result<(), RecommendationError> {
        let request = OpenAIEmbeddingsRequest {
            input,
            model: "text-embedding-3-large".to_owned(),
            // 2d???
            dimensions: Some(2),
        };

        match self.openai_api_client.embeddings(&request).await {
            Ok(r) => {
                let embedding = r.body.data.first();
                if embedding.is_none() {
                    return Err(anyhow::Error::msg(format!(
                        "no embedding returned for {proposition_id}"
                    ))
                    .into());
                }

                let model = offer_embeddings::ActiveModel {
                    proposition_id: sea_orm::ActiveValue::Set(proposition_id),
                    embeddings: sea_orm::ActiveValue::Set(PgVector::from(
                        embedding.unwrap().embedding.clone(),
                    )),
                };

                let conflict = if force {
                    OnConflict::column(offer_embeddings::Column::PropositionId)
                        .update_column(offer_embeddings::Column::Embeddings)
                        .to_owned()
                } else {
                    OnConflict::column(offer_embeddings::Column::PropositionId)
                        .do_nothing()
                        .to_owned()
                };

                offer_embeddings::Entity::insert(model)
                    .on_conflict(conflict)
                    .exec_without_returning(&self.db)
                    .await?;

                tracing::info!("created embedding for: {proposition_id}");
            }
            Err(e) => {
                return Err(
                    anyhow::Error::msg(format!("error in generating embedding: {e}")).into(),
                );
            }
        };

        Ok(())
    }

    async fn refresh_all_embeddings_internal(&self) -> Result<(), RecommendationError> {
        let offer_details = offer_details::Entity::find()
            .filter(
                Condition::any().add(
                    offer_details::Column::PropositionId.not_in_subquery(
                        Query::select()
                            .column(offer_embeddings::Column::PropositionId)
                            .from(offer_embeddings::Entity)
                            .to_owned(),
                    ),
                ),
            )
            .all(&self.db)
            .await?
            .into_iter()
            .map(|od| (od.proposition_id, od.short_name));

        for (id, embedding_input) in offer_details {
            if let Err(e) = self.refresh_embedding_for(id, embedding_input, false).await {
                tracing::error!("{e}");
            }
        }

        Ok(())
    }
}
