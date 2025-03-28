use entity::{offer_details, offer_embeddings};
use error::RecommendationError;
use itertools::Itertools;
use openai::types::OpenAIEmbeddingsRequest;
use sea_orm::prelude::PgVector;
use sea_orm::sea_query::{OnConflict, Query};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter};
use std::convert::TryInto;
use std::{ops::Deref, sync::Arc};
use tokio::runtime::Handle;
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

    fn fixed_size<T, const N: usize>(v: Vec<T>) -> [T; N] {
        v.try_into().unwrap_or_else(|v: Vec<T>| {
            panic!("Expected a Vec of length {} but it was {}", N, v.len())
        })
    }

    #[instrument(skip(self))]
    pub async fn generate_clusters(&self) -> Result<(), RecommendationError> {
        let _all_embeddings: Vec<(i64, [f32; 2])> = offer_embeddings::Entity::find()
            .all(self.db())
            .await?
            .into_iter()
            .map(|m| (m.proposition_id, Self::fixed_size(m.embeddings.to_vec())))
            .collect_vec();

        let rt = Handle::current();

        rt.spawn_blocking(|| {}).await?;

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn refresh_all_embeddings(&self) -> Result<(), RecommendationError> {
        self.refresh_all_embeddings_internal().await
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
            dimensions: None,
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

        let chunk_size = 20;
        let mut current = 0;
        let total = offer_details.len();

        let mut ft = Vec::new();

        for (id, embedding_text) in offer_details {
            let future = Box::pin(async move {
                if let Err(e) = self.refresh_embedding_for(id, embedding_text, false).await {
                    tracing::error!("{e}");
                };
            });

            ft.push(future);
        }

        for ft in ft.chunks_mut(chunk_size) {
            futures::future::join_all(ft).await;

            current += chunk_size;
            let percentage: f32 = (current as f32 / total as f32) * 100.0;
            tracing::info!("progress: {}/{} -> {:.2}%", current, total, percentage);
        }

        Ok(())
    }
}
