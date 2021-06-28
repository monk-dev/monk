use async_graphql::*;

use crate::models::article::Article;

pub struct Query;

#[Object]
impl Query {
    pub async fn articles(&self, ctx: &Context<'_>) -> Result<Vec<Article>> {
        todo!()
    }
}
