use self::{
    mutations::offers::OffersMutation,
    queries::{
        health::HealthQuery, locations::LocationsQuery, offers::OffersQuery, points::PointsQuery,
    },
};
use async_graphql::{EmptySubscription, MergedObject, Schema};

mod handler;
pub mod mutations;
pub mod queries;
pub use handler::*;

#[derive(Default, MergedObject)]
pub struct QueryRoot(HealthQuery, OffersQuery, PointsQuery, LocationsQuery);

#[derive(Default, MergedObject)]
pub struct MutationRoot(OffersMutation);

pub type FinalSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;
