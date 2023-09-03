use http::{HeaderMap, HeaderValue};

pub trait HeaderMapExtensions {
    fn get_or_default(&self, header: &str, s: &str) -> HeaderValue;
}

impl HeaderMapExtensions for HeaderMap {
    fn get_or_default(&self, header: &str, s: &str) -> HeaderValue {
        self.get(header)
            .unwrap_or(&HeaderValue::from_str(s).unwrap())
            .clone()
    }
}
