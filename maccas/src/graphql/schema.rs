use super::query::QueryRoot;
use async_graphql::{EmptyMutation, EmptySubscription, Schema};

pub type MaccasSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;
