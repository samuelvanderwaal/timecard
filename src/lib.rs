#[macro_use]
extern crate diesel;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel::result::Error as DieselError;
use dotenv::dotenv;
use std::env;
// use chrono::{Local, DateTime, Datelike, Timelike};

pub mod schema;

use schema::entries;

#[derive(Debug, Clone, Queryable)]
pub struct Entry {
    pub id: i32,
    pub start: String,
    pub stop: String,
    pub code: String,
    pub memo: String,
}

#[derive(Debug, Clone, Insertable)]
#[table_name="entries"]
pub struct NewEntry {
    pub start: String,
    pub stop: String,
    pub code: String,
    pub memo: String,
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("Database url must be set!");
    SqliteConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
}

// pub fn write_entry(conn: &SqliteConnection, new_entry: &NewEntry) -> bool {
//     diesel::insert_into(entries::table)
//         .values(new_entry)
//         .execute(conn)
//         .is_ok()
// }

pub fn write_entry(conn: &SqliteConnection, new_entry: &NewEntry) -> Result<usize, DieselError> {
    diesel::insert_into(entries::table)
        .values(new_entry)
        .execute(conn)
}