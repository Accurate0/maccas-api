use futures::lock::Mutex;
use lazy_static::lazy_static;
use rand::{rngs::SmallRng, SeedableRng};
use std::sync::Arc;

lazy_static! {
    pub static ref RNG: Arc<Mutex<SmallRng>> = Arc::new(Mutex::new(SmallRng::from_entropy()));
}
