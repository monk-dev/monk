use async_graphql::*;
use tracing::info;
use uuid::Uuid;

use crate::{connection::DbConn, models::article::Article};

pub struct Query;

#[Object]
impl Query {
    pub async fn article(&self, ctx: &Context<'_>, id: Uuid) -> Result<Article> {
        info!(%id, "getting article");

        let conn = ctx.data::<DbConn>()?.get().await;
        Article::get(&conn, &id).map_err(Into::into)
    }

    pub async fn articles(&self, ctx: &Context<'_>) -> Result<Vec<Article>> {
        info!("getting articles");

        let conn = ctx.data::<DbConn>()?.get().await;
        Article::all(&conn).map_err(Into::into)
    }
}
