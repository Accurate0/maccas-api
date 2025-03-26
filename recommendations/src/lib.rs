pub struct Health {}

impl Health {
    pub fn path() -> &'static str {
        "health"
    }
}

pub struct GenerateEmbeddings {}

impl GenerateEmbeddings {
    pub fn path() -> &'static str {
        "generate"
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GenerateEmbeddingsFor;

impl GenerateEmbeddingsFor {
    pub fn path(proposition_id: i64) -> String {
        format!("generate/{proposition_id}")
    }
}


