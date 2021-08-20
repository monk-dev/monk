use async_graphql::{EmptySubscription, Schema};

use crate::{mutation::Mutation, query::Query};

pub type MonkSchema = Schema<Query, Mutation, EmptySubscription>;
