use crate::event_manager::EventManager;

#[derive(Clone)]
pub struct AppState {
    pub event_manager: EventManager,
}
