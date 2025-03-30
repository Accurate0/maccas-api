use sea_orm::prelude::Uuid;

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

    pub fn template_path() -> String {
        format!("/{}", Self::path())
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GenerateEmbeddingsFor;

impl GenerateEmbeddingsFor {
    pub fn path(proposition_id: i64) -> String {
        format!("generate/{proposition_id}")
    }

    pub fn template_path() -> &'static str {
        "/generate/{id}"
    }
}

pub struct GenerateClusters {}

impl GenerateClusters {
    pub fn path() -> &'static str {
        "generate/clusters"
    }

    pub fn template_path() -> String {
        format!("/{}", Self::path())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GenerateClusterScores {
    pub user_ids: Vec<Uuid>,
}

impl GenerateClusterScores {
    pub fn path() -> &'static str {
        "generate/cluster/scores"
    }

    pub fn template_path() -> String {
        format!("/{}", Self::path())
    }
}
