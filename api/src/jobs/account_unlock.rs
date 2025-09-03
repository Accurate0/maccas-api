use super::{error::JobError, Job, JobContext};
use entity::account_lock;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct AccountUnlockJob;

#[async_trait::async_trait]
impl Job for AccountUnlockJob {
    fn name(&self) -> String {
        "account_unlock".to_owned()
    }

    // FIXME: this fixes the symptom but not the cause of the issue
    async fn execute(
        &self,
        context: &JobContext,
        _cancellation_token: CancellationToken,
    ) -> Result<(), JobError> {
        let now = chrono::offset::Utc::now();
        let res = account_lock::Entity::delete_many()
            .filter(account_lock::Column::UnlockAt.lte(now))
            .exec(context.database)
            .await?;

        tracing::info!("unlocked {} accounts", res.rows_affected);

        Ok(())
    }
}
