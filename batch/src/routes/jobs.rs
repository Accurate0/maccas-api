use crate::{
    jobs::{job_scheduler::Message, IntrospectedJobDetails},
    types::{ApiState, AppError},
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use entity::job_history;
use reqwest::StatusCode;
use sea_orm::{EntityTrait, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct TaskQueueResult {
    pub seconds_until_next: u64,
    pub name: String,
}

#[derive(Serialize)]
pub struct GetJobsResponse {
    pub current_jobs: Vec<IntrospectedJobDetails>,
    pub history: Vec<job_history::Model>,
    pub task_queue: Vec<TaskQueueResult>,
}

#[derive(Deserialize)]
pub struct Filter {
    pub limit: Option<u64>,
}

pub async fn get_jobs(
    State(state): State<ApiState>,
    query: Query<Filter>,
) -> Result<Json<GetJobsResponse>, AppError> {
    let current_jobs = state.job_scheduler.introspect();
    let history = job_history::Entity::find()
        .limit(Some(query.limit.unwrap_or(50)))
        .order_by_desc(job_history::Column::CreatedAt)
        .all(state.job_scheduler.db());

    let ((current_jobs, task_queue), history) = futures::future::join(current_jobs, history).await;

    Ok(Json(GetJobsResponse {
        current_jobs,
        history: history?,
        task_queue: task_queue
            .into_iter()
            .filter_map(|x| match x.value {
                Message::RunJob { name, .. } => Some(TaskQueueResult {
                    seconds_until_next: x.delay_util.as_secs(),
                    name,
                }),
                _ => None,
            })
            .collect(),
    }))
}

pub async fn run_job(
    State(state): State<ApiState>,
    Path(job_name): Path<String>,
) -> Result<StatusCode, AppError> {
    state.job_scheduler.run_job(&job_name).await?;
    Ok(StatusCode::NO_CONTENT)
}
