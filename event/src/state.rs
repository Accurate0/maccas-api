use crate::{event_manager::EventManager, settings::Settings};

#[derive(Clone)]
pub struct AppState {
    pub event_manager: EventManager,
    pub settings: Settings,
}
