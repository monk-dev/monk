use tantivy::schema::*;

pub static SCHEMA_VERSION: &'static str = "0.0.0";

pub const ID: Field = Field::from_field_id(0);
pub const NAME: Field = Field::from_field_id(1);
pub const URL: Field = Field::from_field_id(2);
pub const COMMENT: Field = Field::from_field_id(3);
pub const BODY: Field = Field::from_field_id(4);
pub const TITLE: Field = Field::from_field_id(5);
pub const EXTRA: Field = Field::from_field_id(6);
pub const FOUND: Field = Field::from_field_id(7);

pub fn current_schema() -> Schema {
    let mut builder = Schema::builder();

    let _ = builder.add_text_field("id", STORED | STRING);
    let _ = builder.add_text_field("name", TEXT);
    let _ = builder.add_text_field("url", TEXT);
    let _ = builder.add_text_field("comment", TEXT);
    let _ = builder.add_text_field("body", TEXT);
    let _ = builder.add_text_field("title", TEXT);
    let _ = builder.add_text_field("extra", TEXT);
    let _ = builder.add_date_field("found", FAST | INDEXED);

    builder.build()
}
