use crate::constants;
use http::{HeaderValue, Request};

pub trait RequestExtensions<T> {
    fn get_correlation_id(&self) -> HeaderValue;
}

impl<T: std::fmt::Debug> RequestExtensions<T> for Request<T> {
    fn get_correlation_id(&self) -> HeaderValue {
        self.headers()
            .get(constants::CORRELATION_ID_HEADER)
            .unwrap_or(&HeaderValue::from_str(&crate::util::get_uuid()).unwrap())
            .clone()
    }
}
