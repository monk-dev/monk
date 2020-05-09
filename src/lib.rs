pub mod error;
pub mod schema;

// use anyhow::Result;
use structopt::StructOpt;

use tantivy::{
    chrono::prelude::*,
    collector::TopDocs,
    doc,
    query::{AllQuery, QueryParser},
    schema::*,
    DocAddress, Index,
};

use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq, StructOpt)]
pub enum Args {
    /// Add an item to the database
    Add {
        /// The name of the item
        // #[structopt()]
        name: String,
        /// The uri of the item
        #[structopt(short, long)]
        uri: Option<String>,
        /// The body of the item
        #[structopt(short, long)]
        body: Option<String>,
        /// The type of item: article, project, newsletter, forum, repo
        #[structopt(name = "type", short, long)]
        ty: Option<String>,
        /// Any associated comment for the item
        #[structopt(short, long)]
        comment: Option<String>,
    },
    /// List all items in the database
    List {
        #[structopt(short, long, default_value = "0")]
        count: usize,
    },
}

pub fn run(args: Args) -> Result<(), Error> {
    #![allow(unused_must_use)]
    std::fs::create_dir("tantivy");

    let mut builder: SchemaBuilder = Schema::builder();
    // let uri = builder.add_text_field("uri", TEXT | STORED);
    let name = builder.add_text_field("name", TEXT | STORED);
    let uri = builder.add_text_field("comment", STRING);
    let type_ = builder.add_text_field("type", STRING | STORED);
    let body = builder.add_text_field("body", TEXT | STORED);
    let comment = builder.add_text_field("comment", TEXT);
    let discovered = builder.add_date_field("discovered", INDEXED | STORED);

    // dated (for specific date to be read on / associated event or time)
    // stars?
    // Comments?
    // body for searching
    // title?
    let schema = builder.build();

    let dir = tantivy::directory::MmapDirectory::open("tantivy")?;
    let index = Index::open_or_create(dir, schema.clone())?;

    let now = Utc::now();

    match args {
        Args::Add {
            name: item_name,
            uri: item_uri,
            body: item_body,
            ty,
            comment: item_comment,
        } => {
            let mut index_writer = index.writer(3000000)?;

            // let ty = ty.unwrap_or_else(|| "article".into());

            let mut doc = Document::new();

            doc.add_text(name, &item_name);

            if let Some(ref item_uri) = item_uri {
                doc.add_text(uri, item_uri);
            }

            if let Some(ref item_body) = item_body {
                doc.add_text(body, item_body);
            }

            if let Some(ref ty) = ty {
                doc.add_text(type_, ty);
            }

            if let Some(ref item_comment) = item_comment {
                doc.add_text(comment, item_comment);
            }

            println!("{:?}", doc);

            index_writer.add_document(doc);
            index_writer.commit()?;
        }
        Args::List { count } => {
            let index_reader = index.reader()?;
            let searcher = index_reader.searcher();

            let count = if count == 0 {
                searcher.num_docs() as usize
            } else {
                count
            };

            if count == 0 {
                println!("there are no items");
                return Ok(());
            }

            let docs: Vec<(f32, DocAddress)> =
                searcher.search(&AllQuery, &TopDocs::with_limit(count))?;

            for (_weight, address) in docs {
                let doc = searcher.doc(address)?;

                print_document(&doc);
            }
        }
    }

    Ok(())
}

fn print_document(doc: &Document) {
    use colored::*;

    let mut output = String::new();

    let mut title_string = String::new();

    if let Some(type_) = doc.get_first(Field::from_field_id(2)) {
        title_string.push_str(match type_.text().unwrap() {
            "article" => "article",
        })
    }

    if let Some(name) = doc.get_first(Field::from_field_id(0)) {
        title_string.push_str(&name.text().unwrap().underline().red().to_string());
        title_string.push_str(": ");
    }

    // let name = doc.get_first(Field::from_field_id(0)).unwrap();
    // let type_ = doc.get_first(Field::from_field_id(1)).unwrap();
    // let body = doc.get_first(Field::from_field_id(2)).unwrap();
    // let discovered = doc.get_first(Field::from_field_id(3)).unwrap();

    // let mut title_string = String::new();

    // if let Some(name_text) = name.text() {
    //     if name_text.is_empty() {
    //         title_string.push_str("N/A");
    //     } else {
    //         title_string.push_str(name_text);
    //     }

    //     title_string.push_str(": ");
    // } else {
    //     title_string.push('^');
    // }

    // let mut title_string = title_string.red().bold().underline().to_string();

    // title_string.push_str(type_.text().unwrap_or_default());

    // let mut body_string = "\t- ".to_string();
    // body_string.push_str(match body.text().unwrap_or_default() {
    //     "" => "N/A",
    //     b => b,
    // });

    // let mut date_found = String::from("\t- ");
    // date_found.push_str(&discovered.date_value().format("%b %e, %Y").to_string());

    // println!("{}\n{}\n{}", title_string, body_string, date_found.blue());
}
