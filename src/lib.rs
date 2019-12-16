#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

use chrono::{Datelike, Duration, Local};
use diesel::dsl;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::sqlite::SqliteConnection;
use dotenv::dotenv;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
// use chrono::{Local, DateTime, Datelike, Timelike};

pub mod schema;

use schema::{entries, projects};

#[derive(Debug, Clone, Queryable, QueryableByName, Deserialize)]
#[table_name = "entries"]
pub struct Entry {
    pub id: i32,
    pub start: String,
    pub stop: String,
    pub week_day: String,
    pub code: String,
    pub memo: String,
}

#[derive(Debug, Clone, Insertable)]
#[table_name = "entries"]
pub struct NewEntry {
    pub start: String,
    pub stop: String,
    pub week_day: String,
    pub code: String,
    pub memo: String,
}

#[derive(Debug, Clone, PartialEq, Queryable, QueryableByName, Deserialize)]
#[table_name = "projects"]
pub struct Project {
    pub id: i32,
    pub code: String,
    pub name: String,
}

lazy_static! {
    static ref WEEKDAYS: HashMap<String, i64> = vec![
        ("Sun".to_owned(), 0),
        ("Mon".to_owned(), 1),
        ("Tue".to_owned(), 2),
        ("Wed".to_owned(), 3),
        ("Thu".to_owned(), 4),
        ("Fri".to_owned(), 5),
        ("Sat".to_owned(), 6),
    ]
    .into_iter()
    .collect();
}

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("Database url must be set!");
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

// pub fn write_entry(conn: &SqliteConnection, new_entry: &NewEntry) -> bool {
//     diesel::insert_into(entries::table)
//         .values(new_entry)
//         .execute(conn)
//         .is_ok()
// }

// Debug
pub fn write_entry(conn: &SqliteConnection, new_entry: &NewEntry) -> Result<usize, DieselError> {
    diesel::insert_into(entries::table)
        .values(new_entry)
        .execute(conn)
}

pub fn entries_this_week(conn: &SqliteConnection) -> Vec<Entry> {
    let week_day: String = Local::today().weekday().to_string();
    let offset = *WEEKDAYS.get(&week_day).expect("Day does not exist!");
    let week_beginning = Local::today() - Duration::days(offset);
    let week_entries: Vec<Entry> = dsl::sql_query(format!(
        "SELECT * FROM entries WHERE start > '{}'",
        week_beginning
    ))
    .load(conn)
    .unwrap();
    week_entries
}

fn read_projects(conn: &SqliteConnection) -> Vec<Project> {
    dsl::sql_query("SELECT * FROM projects;")
        .load(conn)
        .unwrap()
}

pub fn create_weekly_report(conn: &SqliteConnection) {
    use schema::entries::dsl::*;

    // let projects = read_projects(conn);
    let day_of_week: String = Local::today().weekday().to_string();
    let offset = *WEEKDAYS.get(&day_of_week).expect("Day does not exist!");
    let week_beginning = Local::today() - Duration::days(offset);

    // for project in projects {
        // let sum: i32 = dsl::sql_query(format!(
        // "select sum((strftime('%s', stop) - strftime('%s', start))/3600) from entries where code='{}';", project.code))
    // }
    let sum: i32 = dsl::sql_query(format!(
        "select sum((strftime('%s', stop) - strftime('%s', start))/3600) from entries where code='{}';", "19-000-01"))
        .load(conn)
        .unwrap();

    println!("{:?}", sum);

    // let results = entries
    //     .filter(code.eq("19-000-1"))
    //     .load::<Entry>(conn)
    //     .expect("error");
    // println!("{:?}", results);
}