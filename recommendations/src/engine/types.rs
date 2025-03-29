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

impl ClusteringRequest {
    pub fn path() -> &'static str {
        "clusters"
    }
}

#[derive(serde::Deserialize, Debug)]
pub struct ClusteringResponse(pub HashMap<i64, Vec<String>>);

pub struct ClusteringHealthRequest;
impl ClusteringHealthRequest {
    pub fn path() -> &'static str {
        "health"
    }
}
