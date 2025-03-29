use crate::settings::Settings;
use entity::{offer_details, offer_embeddings};
use error::RecommendationError;
use itertools::Itertools;
use openai::types::OpenAIEmbeddingsRequest;
use reqwest::{Method, StatusCode};
use sea_orm::prelude::PgVector;
use sea_orm::sea_query::{OnConflict, Query};
use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect};
use std::{ops::Deref, sync::Arc};
use tracing::instrument;
use types::{
    ClusteringHealthRequest, ClusteringRequest, ClusteringRequestEmbedding, ClusteringResponse,
};

mod error;
mod types;

#[derive(Clone, Debug)]
pub struct RecommendationEngine {
    inner: Arc<EngineInner>,
}

#[derive(Debug)]
pub struct EngineInner {
    db: DatabaseConnection,
    settings: Settings,
    openai_api_client: openai::ApiClient,
}

impl Deref for RecommendationEngine {
    type Target = EngineInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RecommendationEngine {
    pub fn new(
        db: DatabaseConnection,
        openai_api_client: openai::ApiClient,
        settings: Settings,
    ) -> Self {
        Self {
            inner: Arc::new(EngineInner {
                settings,
                db,
                openai_api_client,
            }),
        }
    }

    pub async fn is_healthy(&self) -> Result<bool, RecommendationError> {
        let is_db_ok = self.db.ping().await.ok().is_some();

        let http_client = base::http::get_http_client()?;
        let url = format!(
            "{}/{}",
            self.settings.clustering_api_base,
            ClusteringHealthRequest::path()
        );

        let is_clustering_ok = http_client
            .request(Method::GET, url)
            .send()
            .await?
            .error_for_status()?
            .status()
            == StatusCode::NO_CONTENT;

        tracing::info!("db: {is_db_ok}, clustering: {is_clustering_ok}");

        Ok(is_db_ok && is_clustering_ok)
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    #[instrument(skip(self))]
    pub async fn generate_clusters(&self) -> Result<(), RecommendationError> {
        let embeddings = offer_embeddings::Entity::find()
            .all(self.db())
            .await?
            .into_iter()
            .map(|m| ClusteringRequestEmbedding {
                name: m.name,
                embedding: m.embeddings.to_vec(),
            });

        let http_client = base::http::get_http_client()?;
        let url = format!(
            "{}/{}",
            self.settings.clustering_api_base,
            ClusteringRequest::path()
        );

        let response = http_client
            .request(Method::POST, url)
            .json(&ClusteringRequest {
                embeddings: embeddings.collect_vec(),
            })
            .send()
            .await?
            .error_for_status()?
            .json::<ClusteringResponse>()
            .await?;

        tracing::info!("{response:?}");

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn refresh_all_embeddings(&self) -> Result<(), RecommendationError> {
        self.refresh_all_embeddings_internal().await
    }

    #[instrument(skip(self))]
    pub async fn refresh_embedding_for(
        &self,
        input: String,
        force: bool,
    ) -> Result<(), RecommendationError> {
        let request = OpenAIEmbeddingsRequest {
            input: input.to_owned(),
            model: "text-embedding-3-large".to_owned(),
            // 2d???
            dimensions: None,
        };

        match self.openai_api_client.embeddings(&request).await {
            Ok(r) => {
                let embedding = r.body.data.first();
                if embedding.is_none() {
                    return Err(anyhow::Error::msg("no embedding returned").into());
                }

                let model = offer_embeddings::ActiveModel {
                    name: sea_orm::ActiveValue::Set(input),
                    embeddings: sea_orm::ActiveValue::Set(PgVector::from(
                        embedding.unwrap().embedding.clone(),
                    )),
                };

                let conflict = if force {
                    OnConflict::column(offer_embeddings::Column::Name)
                        .update_column(offer_embeddings::Column::Embeddings)
                        .to_owned()
                } else {
                    OnConflict::column(offer_embeddings::Column::Name)
                        .do_nothing()
                        .to_owned()
                };

                offer_embeddings::Entity::insert(model)
                    .on_conflict(conflict)
                    .exec_without_returning(&self.db)
                    .await?;

                tracing::info!("created embedding");
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
            .distinct_on([offer_details::Column::ShortName])
            .filter(
                Condition::any().add(
                    offer_details::Column::ShortName.not_in_subquery(
                        Query::select()
                            .column(offer_embeddings::Column::Name)
                            .from(offer_embeddings::Entity)
                            .to_owned(),
                    ),
                ),
            )
            .all(&self.db)
            .await?
            .into_iter()
            .map(|od| od.short_name);

        let chunk_size = 10;
        let mut current = 0;
        let total = offer_details.len();

        let mut future_list = Vec::new();

        for embedding_text in offer_details {
            let future = Box::pin(async move {
                if let Err(e) = self.refresh_embedding_for(embedding_text, false).await {
                    tracing::error!("{e}");
                };
            });

            future_list.push(future);
        }

        for future_chunk in future_list.chunks_mut(chunk_size) {
            let size = future_chunk.len();
            futures::future::join_all(future_chunk).await;

            current += size;
            let percentage: f32 = (current as f32 / total as f32) * 100.0;
            tracing::info!("progress: {}/{} -> {:.2}%", current, total, percentage);
        }

        Ok(())
    }
}
