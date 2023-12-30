pub mod create_task;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum Task {
    Cleanup { offer_id: String },
}
