use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct ClusteringRequestEmbedding {
    pub name: String,
    pub embedding: Vec<f32>,
}

#[derive(serde::Serialize)]
pub struct ClusteringRequest {
    pub embeddings: Vec<ClusteringRequestEmbedding>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ClusteringResponse(pub HashMap<i64, Vec<String>>);
