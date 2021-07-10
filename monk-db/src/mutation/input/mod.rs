use async_graphql::InputObject;
use url::Url;
use uuid::Uuid;

#[derive(InputObject)]
pub struct CreateArticleInput {
    pub name: String,
    pub description: Option<String>,
    pub url: Option<Url>,
    pub tags: Option<Vec<Uuid>>,
}

#[derive(InputObject)]
pub struct UpdateArticleInput {
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<Url>,
}

#[derive(InputObject)]
pub struct DeleteArticleInput {
    pub id: Uuid,
}
