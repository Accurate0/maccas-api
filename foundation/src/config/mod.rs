#[cfg(feature = "aws")]
mod load_from_s3;
#[cfg(feature = "aws")]
pub mod sources;
#[cfg(feature = "aws")]
pub use load_from_s3::load_config_from_s3;
