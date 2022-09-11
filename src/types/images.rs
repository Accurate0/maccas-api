use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct OfferImageBaseName {
    pub original: String,
    pub new: String,
}
