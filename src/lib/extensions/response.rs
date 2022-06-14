use http::Response;

pub trait ResponseExtensions<T> {
    fn log(&self);
}

impl<T: std::fmt::Debug> ResponseExtensions<T> for Response<T> {
    fn log(&self) {
        log::info!("status: {}", self.status());
        log::info!("headers: {:?}", self.headers());
        log::info!("body: {:?}", self.body());
    }
}
