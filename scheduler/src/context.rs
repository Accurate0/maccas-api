use crate::settings::Settings;
use reqwest_middleware::ClientWithMiddleware;

#[derive(Debug)]
pub struct SchedulerContext {
    pub http_client: ClientWithMiddleware,
    pub settings: Settings,
}

impl Default for SchedulerContext {
    fn default() -> Self {
        Self {
            settings: Default::default(),
            http_client: base::http::get_http_client().expect("must create http client"),
        }
    }
}
