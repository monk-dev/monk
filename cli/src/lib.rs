pub mod error;
pub mod schema;
pub mod settings;

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
    // #![allow(unused_must_use)]
    // std::fs::create_dir("tantivy");

    Ok(())
}

// Document { field_values: [
// FieldValue { field: Field(0), value: Str("Cool Article") },
// FieldValue { field: Field(1), value: Str("file:://cool.article") },
// FieldValue { field: Field(3), value: Str("This is the coolest thing ever!") },
// FieldValue { field: Field(2), value: Str("article") },
// FieldValue { field: Field(4), value: Str("Should really check this out") }] }

fn print_document(doc: &Document) {
    use colored::*;
    use std::fmt::Write;

    // println!("{:?}", doc);

    let mut title_string = String::new();
    if let Some(type_) = doc.get_first(Field::from_field_id(2)) {
        title_string.push_str(match type_.text().unwrap() {
            "article" => "Article ",
            t => t,
        })
    }

    if let Some(name) = doc.get_first(Field::from_field_id(0)) {
        writeln!(
            &mut title_string,
            "{}:",
            name.text().unwrap().underline().red()
        )
        .unwrap();
    } else {
        println!("No title");
    }

    let mut body_string = String::new();

    if let Some(body) = doc.get_first(Field::from_field_id(3)) {
        writeln!(&mut body_string, "{} {}", "\t", body.text().unwrap()).unwrap();
    } else {
        println!("No body");
    }

    if let Some(comment) = doc.get_first(Field::from_field_id(4)) {
        writeln!(
            &mut body_string,
            "{} {}",
            "\t",
            comment.text().unwrap().green()
        )
        .unwrap();
    } else {
        println!("No comment");
    }

    if let Some(uri) = doc.get_first(Field::from_field_id(1)) {
        writeln!(&mut body_string, "{} {}", "\t", uri.text().unwrap().blue()).unwrap();
    } else {
        println!("No URI");
    }

    if let Some(discovered) = doc.get_first(Field::from_field_id(5)) {
        writeln!(
            &mut body_string,
            "{} {}",
            "\t",
            discovered.date_value().format("%a %d, %Y").to_string()
        )
        .unwrap();
    } else {
        println!("No time");
    }

    print!("{}{}", title_string, body_string);
}
