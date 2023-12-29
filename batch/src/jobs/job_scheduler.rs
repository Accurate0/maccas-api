use super::{error::JobError, Job, JobContext, JobDetails, JobState, JobType, RunningState};
use entity::jobs;
use sea_orm::{
    prelude::Uuid, sea_query::OnConflict, ActiveModelTrait, ColumnTrait, DatabaseConnection,
    EntityTrait, QueryFilter, Set,
};
use std::{collections::HashMap, fmt::Debug, ops::ControlFlow, sync::Arc, time::Duration};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex, RwLock,
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Level};

pub(crate) enum Message {
    JobFinished { name: String },
    Init,
    RunPending(String),
}

impl Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::JobFinished { name } => {
                write!(f, "JobFinished: {}", name)
            }
            Self::Init => write!(f, "Init"),
            Self::RunPending(arg0) => {
                write!(f, "RunPending: {}", arg0)
            }
        }
    }
}

struct Queue {
    pub tx: UnboundedSender<Message>,
    pub rx: Mutex<UnboundedReceiver<Message>>,
}

pub struct JobSchedulerInner {
    cancellation_token: CancellationToken,
    jobs: Arc<RwLock<HashMap<String, JobDetails>>>,
    db: DatabaseConnection,
    task_queue: Queue,
}

#[derive(Clone)]
pub struct JobScheduler(Arc<JobSchedulerInner>);

impl JobScheduler {
    pub fn new(db: DatabaseConnection) -> Self {
        let channel = unbounded_channel::<Message>();

        JobScheduler(
            JobSchedulerInner {
                db,
                cancellation_token: Default::default(),
                jobs: Default::default(),
                task_queue: Queue {
                    tx: channel.0,
                    rx: Mutex::new(channel.1),
                },
            }
            .into(),
        )
    }

    pub async fn add_scheduled<T>(&self, job: T, schedule: cron::Schedule) -> &Self
    where
        T: Job + 'static,
    {
        let mut jobs = self.0.jobs.write().await;
        jobs.insert(
            job.name(),
            JobDetails::new(Arc::new(job), JobType::Schedule(schedule)),
        );

        self
    }

    pub async fn init(&self) -> Result<(), JobError> {
        let jobs = self.0.jobs.read().await;
        for (name, _) in jobs.iter() {
            let id = Uuid::new_v4();

            let job_model = jobs::ActiveModel {
                id: Set(id),
                name: Set(name.clone()),
                ..Default::default()
            };

            jobs::Entity::insert(job_model)
                .on_conflict(
                    OnConflict::columns([jobs::Column::Name])
                        .do_nothing()
                        .to_owned(),
                )
                .exec_without_returning(&self.0.db)
                .await?;
        }

        self.0.task_queue.tx.send(Message::Init)?;

        Ok(())
    }

    pub async fn shutdown(&self) {
        let mut jobs = self.0.jobs.write().await;
        let mut cancellation_futures = Vec::new();

        for (_, job_details) in jobs.iter_mut() {
            match job_details.state {
                JobState::Stopped => {}
                JobState::Running(ref mut state) => {
                    state.cancellation_token.cancel();
                    cancellation_futures.push(&mut state.handle)
                }
            }
        }

        futures::future::join_all(cancellation_futures).await;
        tracing::info!("shutting down job scheduler");
        self.0.cancellation_token.cancel()
    }

    pub async fn run(&self) -> JoinHandle<std::result::Result<(), JobError>> {
        let cloned_self = self.clone();
        tokio::spawn(async move {
            while let ControlFlow::Continue(_) = cloned_self.tick().await? {}
            Ok(())
        })
    }

    async fn handle_message(&self, message: Message) -> Result<(), JobError> {
        match message {
            Message::JobFinished { name } => {
                tracing::info!("received message for job finished: {}", name);
                let mut jobs = self.0.jobs.write().await;
                match jobs.get_mut(&name) {
                    Some(job_details) => {
                        job_details.state = JobState::Stopped;
                        let job_model = jobs::Entity::find()
                            .filter(jobs::Column::Name.eq(name.clone()))
                            .one(&self.0.db)
                            .await?
                            .unwrap();

                        let last_execution = job_model.last_execution;
                        let job_type = &job_details.job_type;
                        let time_now = chrono::offset::Utc::now();

                        // if no last execution, execute immediately
                        let next = match job_type {
                            JobType::Schedule(schedule) => match last_execution {
                                Some(t) => schedule.after(&t.and_utc()).next(),
                                None => Some(time_now),
                            }
                            .unwrap(),
                        };

                        tracing::info!("time now: {}", time_now);
                        tracing::info!("next scheduled run: {}", next);

                        let time_until = if time_now > next {
                            Duration::ZERO
                        } else {
                            (next - time_now).to_std()?
                        };

                        let tx_channel = self.0.task_queue.tx.clone();
                        // setup next wake up for this job
                        tokio::spawn(async move {
                            tokio::time::sleep(time_until).await;
                            tx_channel.send(Message::RunPending(job_model.name))
                        });
                    }
                    None => tracing::error!(
                        "could not find job by name for state to set state to finished: {}",
                        name
                    ),
                }
            }

            Message::RunPending(name) => {
                tracing::info!("run pending task {}", name);
                let mut jobs = self.0.jobs.write().await;

                let job_model = jobs::Entity::find()
                    .filter(jobs::Column::Name.eq(name.clone()))
                    .one(&self.0.db)
                    .await?
                    .unwrap();

                let job_details = jobs.get_mut(&name).unwrap();
                let time_now = chrono::offset::Utc::now();

                let cancellation_token = CancellationToken::new();
                let cancellation_token_cloned = cancellation_token.clone();
                let job = job_details.job.clone();

                let running = match job_details.state {
                    JobState::Stopped => false,
                    JobState::Running(_) => true,
                };

                if !running {
                    tracing::info!("triggered job {}", name);
                    let context = JobContext::new(self.0.db.clone(), job_model.id);
                    let span = tracing::span!(parent: None, Level::INFO, "job", name);
                    let tx_channel = self.0.task_queue.tx.clone();
                    let task_name = name.clone();

                    let handle = tokio::spawn(
                        async move {
                            job.execute(&context, cancellation_token_cloned).await;
                            job.cleanup(&context).await;
                            if let Err(e) =
                                tx_channel.send(Message::JobFinished { name: task_name })
                            {
                                tracing::error!("error sending finished message: {}", e)
                            };
                        }
                        .instrument(span),
                    );

                    job_details.state = JobState::Running(RunningState {
                        cancellation_token,
                        handle,
                    });

                    jobs::ActiveModel {
                        id: Set(job_model.id),
                        name: Set(name.clone()),
                        last_execution: Set(Some(time_now.naive_utc())),
                        ..Default::default()
                    }
                    .update(&self.0.db)
                    .await?;
                };
            }

            Message::Init => {
                let jobs = self.0.jobs.read().await;
                tracing::info!("initializing scheduler with required tasks");
                for (name, job_details) in jobs.iter() {
                    let job_model = jobs::Entity::find()
                        .filter(jobs::Column::Name.eq(name.clone()))
                        .one(&self.0.db)
                        .await?
                        .unwrap();

                    let last_execution = job_model.last_execution;
                    let job_type = &job_details.job_type;
                    let time_now = chrono::offset::Utc::now();

                    // if no last execution, execute immediately
                    let next = match job_type {
                        JobType::Schedule(schedule) => match last_execution {
                            Some(t) => schedule.after(&t.and_utc()).next(),
                            None => Some(time_now),
                        }
                        .unwrap(),
                    };

                    tracing::info!("time now: {}", time_now);
                    tracing::info!("next scheduled run: {}", next);

                    let time_until = if time_now > next {
                        Duration::ZERO
                    } else {
                        (next - time_now).to_std()?
                    };

                    let tx_channel = self.0.task_queue.tx.clone();
                    tracing::info!("task run in {:?}", time_until);
                    // setup next wake up for jobs
                    tokio::spawn(async move {
                        if time_until.is_zero() {
                            tx_channel.send(Message::RunPending(job_model.name))
                        } else {
                            tokio::time::sleep(time_until).await;
                            tx_channel.send(Message::RunPending(job_model.name))
                        }
                    });
                }
            }
        }

        Ok::<(), JobError>(())
    }

    async fn tick(&self) -> Result<ControlFlow<()>, JobError> {
        let cancellation_token = &self.0.cancellation_token;
        let mut task_queue = self.0.task_queue.rx.lock().await;

        tokio::select! {
            _ = cancellation_token.cancelled() => {
                tracing::info!("ticking cancelled");
                Ok(ControlFlow::Break(()))
            },
            Some(message) = task_queue.recv() => {
                let msg = format!("{:?}", message);
                let span = tracing::span!(Level::INFO, "job", msg);
                self.handle_message(message).instrument(span).await?;
                Ok(ControlFlow::Continue(()))
            }
        }
    }
}
