use tantivy::schema::*;

pub static SCHEMA_VERSION: &'static str = "0.0.0";

pub fn current_schema() -> Schema {
    let mut builder = Schema::builder();

    let _ = builder.add_text_field("id", STORED);
    let _ = builder.add_text_field("name", TEXT);
    let _ = builder.add_text_field("url", TEXT);
    let _ = builder.add_text_field("comment", TEXT);
    let _ = builder.add_date_field("found", FAST);

    builder.build()
}
