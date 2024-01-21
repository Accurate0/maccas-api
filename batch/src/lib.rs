#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Health;

impl Health {
    pub fn path() -> &'static str {
        "health"
    }
}
