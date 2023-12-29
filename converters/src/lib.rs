use chrono::ParseError;
use thiserror::Error;

pub mod offers;
pub mod points;
pub struct Database<T>(pub T);

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("date time parse error has ocurred: `{0}`")]
    DateTimeParseError(#[from] ParseError),
    #[error("unknown error")]
    Unknown,
}
