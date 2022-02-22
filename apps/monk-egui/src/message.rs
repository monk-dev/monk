use monk::types::{
    AddItem, Blob, DeleteItem, EditItem, GetBlob, GetItem, Item, ListItem, Search, SearchResult,
};

pub type Request = RequestMsg;
pub type Response = anyhow::Result<ResponseMsg>;

#[derive(Debug)]
pub enum RequestMsg {
    AddItem(AddItem),
    GetItem(GetItem),
    GetBlob(GetBlob),
    ListItem(ListItem),
    EditItem(EditItem),
    DeleteItem(DeleteItem),
    Search(Search),
}

#[derive(Debug)]
pub enum ResponseMsg {
    Add(Item),
    Get(Option<Item>),
    GetBlob(Option<Blob>),
    List(Vec<Item>),
    Edit(Option<Item>),
    Delete(Option<Item>),
    Search(Vec<SearchResult>),
}
