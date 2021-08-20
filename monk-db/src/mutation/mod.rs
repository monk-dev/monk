pub mod input;
pub mod output;

use async_graphql::*;
use tracing::info;

use crate::{
    connection::DbConn,
    models::{article::Article, tag::Tag},
};

use self::input::{CreateArticleInput, DeleteArticleInput, UpdateArticleInput};

pub struct Mutation;

#[Object]
impl Mutation {
    pub async fn create_article(
        &self,
        ctx: &Context<'_>,
        input: CreateArticleInput,
    ) -> Result<Article> {
        info!(name = %input.name, "create article");

        let conn = ctx.data::<DbConn>()?.get().await;

        let CreateArticleInput {
            name,
            description,
            url,
            tags,
        } = input;

        let mut insert = Article::insert(name);

        if let Some(description) = description {
            insert = insert.description(description);
        }

        if let Some(url) = url {
            insert = insert.url(url);
        }

        if let Some(tags) = tags {
            insert = insert.tags(&tags);
        }

        insert.execute(&conn).map_err(Into::into)
    }

    pub async fn update_article(
        &self,
        ctx: &Context<'_>,
        input: UpdateArticleInput,
    ) -> Result<Article> {
        info!(id = %input.id, "update article");

        let conn = ctx.data::<DbConn>()?.get().await;

        Article::update(&conn, &input).map_err(Into::into)
    }

    pub async fn delete_article(
        &self,
        ctx: &Context<'_>,
        input: DeleteArticleInput,
    ) -> Result<Article> {
        info!(id=%input.id, "delete article");

        let conn = ctx.data::<DbConn>()?.get().await;

        Article::delete(&conn, &input.id).map_err(Into::into)
    }
}
