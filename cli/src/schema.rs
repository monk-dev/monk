use tantivy::schema::*;

pub static SCHEMA_VERSION: &'static str = "0.1.0";

pub fn create_schema() -> Schema {
    let mut builder: SchemaBuilder = Schema::builder();
    // let uri = builder.add_text_field("uri", TEXT | STORED);
    let _class = builder.add_text_field("class", STRING | STORED);
    let _body = builder.add_text_field("body", TEXT | STORED);
    let _discovered = builder.add_date_field("discovered", INDEXED | STORED);

    // dated (for specific date to be read on / associated event or time)
    // stars?
    // Comments?
    // body for searching
    // title?

    builder.build()
}
