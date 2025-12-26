use std::collections::HashMap;

#[derive(serde::Serialize)]
pub struct ClusteringRequestEmbedding<'a> {
    pub name: &'a String,
    pub embedding: &'a [f32],
}

#[derive(serde::Serialize)]
pub struct ClusteringRequest<'a> {
    pub embeddings: Vec<ClusteringRequestEmbedding<'a>>,
}

impl ClusteringRequest<'_> {
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
