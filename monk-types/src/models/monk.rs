use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct AddItem {
    pub name: Option<String>,
    pub url: Option<String>,
    pub body: Option<String>,
    pub comment: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetItem {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GetTag {
    ItemId(String),
    TagId(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GetBlob {
    ItemId(String),
    BlobId(String),
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    pub count: Option<usize>,
    pub tags: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct EditItem {
    pub id: String,
    pub name: Option<String>,
    pub url: Option<String>,
    pub body: Option<String>,
    pub summary: Option<String>,
    pub comment: Option<String>,
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteItem {
    pub id: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkedItems {
    pub id: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreateLink {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteLink {
    pub a: String,
    pub b: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Search {
    pub count: Option<usize>,
    pub query: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub id: Option<String>,
    pub index: bool,
    pub store: bool,
    pub offline: bool,
}
