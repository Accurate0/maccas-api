use async_graphql::InputObject;

#[derive(InputObject, Default)]
pub struct PointsFilterInput {
    pub minimum_points: i64,
}
