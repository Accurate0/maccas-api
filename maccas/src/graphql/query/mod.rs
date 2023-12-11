use self::{deal::DealsQuery, locations::LocationsQuery, points::PointsQuery, user::UserQuery};
use async_graphql::MergedObject;

pub mod deal;
pub mod locations;
pub mod points;
pub mod user;

#[derive(Default, MergedObject)]
pub struct QueryRoot(DealsQuery, UserQuery, PointsQuery, LocationsQuery);
