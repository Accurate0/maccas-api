mod types;

use crate::{routes::Context, types::api::AccountPointMap};
use async_graphql::Object;
use itertools::Itertools;

use self::types::PointsFilterInput;

#[derive(Default)]
pub struct PointsQuery;

#[Object]
impl PointsQuery {
    async fn points<'a>(
        &self,
        gql_ctx: &async_graphql::Context<'a>,
        filter: Option<PointsFilterInput>,
    ) -> Result<Vec<AccountPointMap>, anyhow::Error> {
        let ctx = gql_ctx.data_unchecked::<Context>();
        let point_map = ctx.database.point_repository.get_all_points().await?;
        Ok(point_map
            .iter()
            .filter(|x| {
                filter.is_none()
                    || filter
                        .as_ref()
                        .is_some_and(|f| x.1.total_points > f.minimum_points)
            })
            .map(|(key, value)| AccountPointMap {
                name: key.to_string(),
                total_points: value.total_points,
                life_time_points: value.life_time_points,
            })
            .sorted_by(|a, b| b.total_points.cmp(&a.total_points))
            .collect())
    }
}
