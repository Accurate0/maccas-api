use self::{deals::DealsQuery, user::UserQuery};
use async_graphql::MergedObject;

pub mod deals;
pub mod user;

#[derive(Default, MergedObject)]
pub struct QueryRoot(DealsQuery, UserQuery);
