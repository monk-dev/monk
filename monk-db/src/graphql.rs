use async_graphql::{EmptyMutation, EmptySubscription, Schema};

use crate::query::Query;

pub type MonkSchema = Schema<Query, EmptyMutation, EmptySubscription>;
