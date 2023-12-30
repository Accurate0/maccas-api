use crate::routes::Task;
use base::delay_queue::DelayQueue;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) db: DatabaseConnection,
    pub(crate) task_queue: DelayQueue<Task>,
}
