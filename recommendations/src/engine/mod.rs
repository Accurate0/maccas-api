use crate::settings::Settings;
use entity::sea_orm_active_enums::Action;
use entity::{
    offer_audit, offer_cluster_score, offer_details, offer_embeddings,
    offer_name_cluster_association, recommendations as recommendations_t,
};
use error::RecommendationError;
use itertools::Itertools;
use openai::types::OpenAIEmbeddingsRequest;
use reqwest::{Method, StatusCode};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::{PgVector, Uuid};
use sea_orm::sea_query::{LockBehavior, LockType, OnConflict, Query};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use std::collections::{HashMap, HashSet};
use std::ops::{Add, Mul};
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

    #[instrument(skip(self))]
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

    #[instrument(skip(self, txn))]
    pub async fn generate_recommendations_for_user(
        &self,
        user_id: Uuid,
        txn: &DatabaseTransaction,
    ) -> Result<(), RecommendationError> {
        // lock this users cluster scores
        offer_cluster_score::Entity::find()
            .filter(offer_cluster_score::Column::UserId.eq(user_id))
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
            .all(txn)
            .await?;

        // lock actual recommendation too
        recommendations_t::Entity::find()
            .filter(recommendations_t::Column::UserId.eq(user_id))
            .lock_with_behavior(LockType::Update, LockBehavior::SkipLocked)
            .one(txn)
            .await?;

        let name_to_cluster_id = offer_name_cluster_association::Entity::find()
            .all(txn)
            .await?
            .into_iter()
            .map(|m| (m.name, m.cluster_id))
            .collect::<HashMap<_, _>>();

        let mut offers_used_mapping = HashMap::<_, Vec<_>>::new();
        // TODO: find and remove corresponding remove of these adds
        let audit_filter_conditions = Condition::all()
            .add(offer_audit::Column::UserId.eq(user_id))
            .add(offer_audit::Column::Action.eq(Action::Add));

        let offers_used = offer_audit::Entity::find()
            .filter(audit_filter_conditions)
            .find_also_related(offer_details::Entity)
            .all(txn)
            .await?;

        for (audit, details) in offers_used {
            let details = details.expect("must have details because foreign key");
            offers_used_mapping
                .entry(details.short_name)
                .and_modify(|e| e.push(audit.clone()))
                .or_insert(vec![audit.clone()]);
        }

        // 28 days
        const HALF_LIFE: f64 = 0.975;
        const BASE_SCORE: f64 = 100f64;
        // tl;dr, for each offer name, generate a score based on days since last usage, each day
        // back is multiplied by HALF_LIFE
        let now = chrono::offset::Utc::now().naive_utc();

        let mut cluster_score_map: HashMap<i64, f64> = HashMap::new();

        for (name, audits) in offers_used_mapping {
            let mut score = 0f64;
            let cluster_id = name_to_cluster_id.get(&name);
            if let Some(cluster_id) = cluster_id {
                for audit in audits {
                    let days_since = (now - audit.created_at).num_days();
                    let score_for_deal =
                        BASE_SCORE.mul(HALF_LIFE.powi(days_since.try_into().unwrap()));
                    score += score_for_deal;
                }

                cluster_score_map
                    .entry(*cluster_id)
                    .and_modify(|s| {
                        *s = s.add(score);
                    })
                    .or_insert(score);
            }
        }

        let offer_cluster_score_models =
            cluster_score_map
                .into_iter()
                .map(|(k, v)| offer_cluster_score::ActiveModel {
                    user_id: Set(user_id),
                    cluster_id: Set(k),
                    score: Set(v),
                    ..Default::default()
                });

        offer_cluster_score::Entity::delete_many()
            .filter(offer_cluster_score::Column::UserId.eq(user_id))
            .exec(txn)
            .await?;

        offer_cluster_score::Entity::insert_many(offer_cluster_score_models)
            .exec_without_returning(txn)
            .await?;

        let mut cluster_id_to_names = HashMap::<i64, Vec<String>>::new();
        for (k, v) in name_to_cluster_id {
            cluster_id_to_names
                .entry(v)
                .and_modify(|e| e.push(k.to_owned()))
                .or_insert(vec![k]);
        }

        let top_x_cluster_scores = offer_cluster_score::Entity::find()
            .order_by_desc(offer_cluster_score::Column::Score)
            .filter(offer_cluster_score::Column::UserId.eq(user_id))
            .limit(3)
            .all(txn)
            .await?
            .into_iter()
            .map(|m| m.cluster_id);

        let mut proposition_id_ordered = HashSet::new();
        let mut all_offer_names = HashSet::new();
        // best to worst
        for cluster_id in top_x_cluster_scores {
            tracing::info!("processing cluster_id: {cluster_id}");
            if let Some(offer_names) = cluster_id_to_names.get(&cluster_id) {
                tracing::info!("with {} offer names", offer_names.len());
                let mut filter_cond = Condition::any();
                for offer_name in offer_names {
                    filter_cond = filter_cond.add(offer_details::Column::ShortName.eq(offer_name));
                }

                let proposition_ids = offer_details::Entity::find()
                    .filter(filter_cond)
                    .all(txn)
                    .await?
                    .into_iter()
                    .map(|m| m.proposition_id)
                    .collect_vec();

                tracing::info!("and {} proposition ids", proposition_ids.len());
                proposition_id_ordered.extend(proposition_ids);
                all_offer_names.extend(offer_names.clone());
            } else {
                tracing::warn!("cluster id missing: {cluster_id}");
            }
        }

        recommendations_t::Entity::insert(recommendations_t::ActiveModel {
            user_id: Set(user_id),
            offer_proposition_ids: Set(proposition_id_ordered.into_iter().collect()),
            names: Set(all_offer_names.into_iter().collect()),
            ..Default::default()
        })
        .on_conflict(
            OnConflict::column(recommendations_t::Column::UserId)
                .update_columns([
                    recommendations_t::Column::OfferPropositionIds,
                    recommendations_t::Column::Names,
                ])
                .to_owned(),
        )
        .exec_without_returning(txn)
        .await?;

        Ok(())
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

        let mut models = Vec::new();
        for (cluster_id, names) in &response.0 {
            for name in names {
                let model = offer_name_cluster_association::ActiveModel {
                    name: Set(name.to_owned()),
                    cluster_id: Set(*cluster_id),
                };

                models.push(model);
            }
        }

        tracing::info!("inserting {} associations", models.len());
        offer_name_cluster_association::Entity::insert_many(models)
            .on_conflict(
                OnConflict::column(offer_name_cluster_association::Column::Name)
                    .update_column(offer_name_cluster_association::Column::ClusterId)
                    .to_owned(),
            )
            .exec_without_returning(self.db())
            .await?;

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
