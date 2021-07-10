use async_graphql::{EmptyMutation, EmptySubscription, Schema};

use crate::{mutation::Mutation, query::Query};

pub type MonkSchema = Schema<Query, Mutation, EmptySubscription>;
