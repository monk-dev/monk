#![allow(unreachable_code)]

use async_graphql::{ComplexObject, Result as GQLResult, SimpleObject};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use tracing::info;
use uuid::Uuid;

use crate::Error;

static USER_COLUMNS: &'static str = "id, name, created_at";
static USER_INSERT_COLUMNS: &'static str = "name";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, SimpleObject)]
// #[graphql(complex)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn get(conn: &Connection, id: &Uuid) -> Result<Self, Error> {
        let query = format!("SELECT {} FROM user WHERE id=?", USER_COLUMNS);

        conn.prepare(&query)?
            .query_row([id], User::from_row)
            .map_err(Into::into)
    }

    pub fn insert(name: impl Into<String>) -> InsertUser {
        InsertUser::new(name)
    }

    pub fn from_row(row: &Row) -> Result<Self, rusqlite::Error> {
        Ok(User {
            id: row.get("id")?,
            name: row.get("name")?,
            created_at: row.get("created_at")?,
        })
    }
}

pub struct InsertUser {
    name: String,
}

impl InsertUser {
    pub fn new(name: impl Into<String>) -> InsertUser {
        InsertUser { name: name.into() }
    }

    #[tracing::instrument(skip(self, conn), fields(user.name=%self.name))]
    pub fn execute(self, conn: &Connection) -> Result<User, Error> {
        info!("adding user");

        let query = format!(
            "INSERT INTO user ({}) VALUES (?) RETURNING {}",
            USER_INSERT_COLUMNS, USER_COLUMNS,
        );

        conn.prepare(&query)?
            .query_row(params![self.name], User::from_row)
            .map_err(Into::into)
    }
}
