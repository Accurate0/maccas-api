use crate::{constants, extensions::HeaderMapExtensions, utils::get_uuid};
use http::{header::ORIGIN, HeaderValue, Request};

pub trait RequestExtensions<T> {
    fn get_correlation_id(&self) -> HeaderValue;
    fn log(&self);
}

impl<T: std::fmt::Debug> RequestExtensions<T> for Request<T> {
    fn get_correlation_id(&self) -> HeaderValue {
        self.headers()
            .get(constants::CORRELATION_ID_HEADER)
            .unwrap_or(&HeaderValue::from_str(get_uuid().as_str()).unwrap())
            .clone()
    }

    fn log(&self) {
        let headers = self.headers();
        log::info!("method: {}", self.method());
        log::info!("uri: {}", self.uri());
        log::info!("version: {:?}", self.version());
        log::info!("origin: {:?}", headers.get_or_default(ORIGIN.as_str(), "unknown"));
        log::info!(
            "traceparent: {:?}",
            headers.get_or_default(constants::CORRELATION_ID_HEADER, "none")
        );
        log::info!("body: {:?}", self.body());
    }
}
