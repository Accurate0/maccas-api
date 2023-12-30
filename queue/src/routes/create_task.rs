use super::Task;
use crate::state::AppState;
use actix_web::{post, web, HttpResponse, Responder};
use std::time::Duration;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct CreateTask {
    task: Task,
    delay: Duration,
}

#[post("/task")]
pub(crate) async fn create_task(
    state: web::Data<AppState>,
    request: web::Json<CreateTask>,
) -> impl Responder {
    // persist this to database immediately
    // with a time in came in, and the requested delay
    state
        .task_queue
        .push(request.task.clone(), request.delay)
        .await;

    HttpResponse::NoContent()
}
