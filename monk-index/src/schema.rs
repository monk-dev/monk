use tantivy::schema::{FacetOptions, Field, Schema, FAST, INDEXED, STORED, STRING, TEXT};

pub static SCHEMA_VERSION: &str = "0.0.1";

pub const ID: Field = Field::from_field_id(0);
pub const NAME: Field = Field::from_field_id(1);
pub const URL: Field = Field::from_field_id(2);
pub const COMMENT: Field = Field::from_field_id(3);
pub const BODY: Field = Field::from_field_id(4);
pub const TITLE: Field = Field::from_field_id(5);
pub const EXTRA: Field = Field::from_field_id(6);
pub const FOUND: Field = Field::from_field_id(7);
pub const TAG: Field = Field::from_field_id(8);

pub fn current_schema() -> Schema {
    let mut builder = Schema::builder();

    let _ = builder.add_text_field("id", STORED | STRING);
    let _ = builder.add_text_field("name", STORED | TEXT);
    let _ = builder.add_text_field("url", TEXT);
    let _ = builder.add_text_field("comment", STORED | TEXT);
    let _ = builder.add_text_field("body", STORED | TEXT);
    let _ = builder.add_text_field("title", TEXT);
    let _ = builder.add_text_field("extra", TEXT);
    let _ = builder.add_date_field("found", FAST | INDEXED);
    let _ = builder.add_facet_field("tag", FacetOptions::default().set_stored().set_indexed());

    builder.build()
}
