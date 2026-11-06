use self::{
    mutations::offers::OffersMutation,
    queries::{
        categories::CategoryQuery, health::HealthQuery, locations::LocationsQuery,
        offers::OffersQuery, points::PointsQuery,
    },
};
use async_graphql::{EmptySubscription, MergedObject, Schema, SchemaBuilder};

mod handler;
pub mod mutations;
pub mod queries;
pub mod tracing;
pub use handler::*;
pub mod guard;

#[derive(Default, MergedObject)]
#[graphql(name = "Query")]
pub struct QueryRoot(
    HealthQuery,
    OffersQuery,
    PointsQuery,
    LocationsQuery,
    CategoryQuery,
);

#[derive(Default, MergedObject)]
#[graphql(name = "Mutation")]
pub struct MutationRoot(OffersMutation);

pub type FinalSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

pub fn builder() -> SchemaBuilder<QueryRoot, MutationRoot, EmptySubscription> {
    Schema::build(
        QueryRoot::default(),
        MutationRoot::default(),
        EmptySubscription,
    )
    .enable_federation()
}

#[cfg(test)]
mod tests {
    use async_graphql::SDLExportOptions;

    fn check(sdl: String, relative: &str) {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
        let current = std::fs::read_to_string(&path).unwrap_or_default();

        if current != sdl {
            std::fs::write(&path, &sdl).expect("write schema");
            panic!("{relative} was out of date and has been regenerated, commit it");
        }
    }

    #[test]
    fn schema_is_current() {
        check(super::builder().finish().sdl(), "../web/schema.graphql");
    }

    #[test]
    fn subgraph_is_current() {
        check(
            super::builder()
                .finish()
                .sdl_with_options(SDLExportOptions::new().federation()),
            "../subgraph.graphql",
        );
    }
}
