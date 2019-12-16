#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate lazy_static;

use rusqlite::{params, NO_PARAMS, Connection, Result as SqlResult};
use chrono::{NaiveDateTime, Datelike, Duration, Local};
use std::collections::HashMap;
use prettytable::{Table, Row, Cell};
use indexmap::IndexMap;
use std::env;
use dotenv::dotenv;

#[derive(Debug, Clone)]
pub struct Entry {
    pub id: i32,
    pub start: String,
    pub stop: String,
    pub week_day: String,
    pub code: String,
    pub memo: String,
}

#[derive(Debug, Clone)]
pub struct NewEntry {
    pub start: String,
    pub stop: String,
    pub week_day: String,
    pub code: String,
    pub memo: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub id: i32,
    pub code: String,
    pub name: String,
}

static DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

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

pub fn establish_connection() -> Connection {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("Database url must be set!");
    let conn = Connection::open(db_url).unwrap();

    // Create tables if they don't already exist.
    conn.execute("CREATE TABLE IF NOT EXISTS entries (
        id INTEGER PRIMARY KEY,
        start TEXT NOT NULL,
        stop TEXT NOT NULL,
        week_day TEXT NOT NULL,
        code TEXT NOT NULL,
        memo TET NOT NULL
        )", NO_PARAMS).expect("Connection execution error!");

    conn.execute("CREATE TABLE IF NOT EXISTS proejcts (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        code TEXT NOT NULL
        )", NO_PARAMS).expect("Connection execution error!");
    conn 
}

// Debug
pub fn write_entry(conn: &Connection, new_entry: &NewEntry) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO entries (start, stop, week_day, code, memo)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![new_entry.start, new_entry.stop, new_entry.week_day, new_entry.code, new_entry.memo],
            )?;
    Ok(())
}

// pub fn entries_this_week(conn: &Connection) -> Vec<Entry> {
//     let week_day: String = Local::today().weekday().to_string();
//     let offset = *WEEKDAYS.get(&week_day).expect("Day does not exist!");
//     let week_beginning = Local::today() - Duration::days(offset);
//     let week_entries: Vec<Entry> = dsl::sql_query(format!(
//         "SELECT * FROM entries WHERE start > '{}'",
//         week_beginning
//     ))
//     .load(conn)
//     .unwrap();
//     week_entries
// }

fn read_projects(conn: &Connection) -> SqlResult<Vec<Project>> {
    let query = "SELECT * FROM projects";
    let mut stmt = conn.prepare(query)?;
    let project_iter = stmt.query_map(params![], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            code: row.get(2)?,
        })
    })?;
    let projects: Vec<Project> = project_iter.into_iter().map(|p| p.unwrap()).collect();
    Ok(projects)

}

pub fn create_weekly_report(conn: &Connection) -> SqlResult<()> {
    let projects = read_projects(conn)?;
    let day_of_week: String = Local::today().weekday().to_string();
    let offset = *WEEKDAYS.get(&day_of_week).expect("Day does not exist!");
    let week_beginning = Local::today() - Duration::days(offset);
    let parse_from_str = NaiveDateTime::parse_from_str;

    // Set up table for printing.
    let mut table = Table::new();
    table.add_row(row![Fb => "Project", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);
    
    for project in projects {
        let query = format!("SELECT start, stop, week_day FROM entries WHERE code='{}' AND start > '{}';", 
            project.code, week_beginning);
        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt.query(NO_PARAMS)?;

        // Set up hashmap to track hours per week day.
        let mut week_hours: IndexMap<String, f64> = IndexMap::new();
        let week_days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        for day in week_days.iter() {
            week_hours.insert((*day).to_owned(), 0.0);
        }

        // Set up row and add project code.
        let mut cells: Vec<Cell> = Vec::new();
        cells.push(Cell::new(&project.code));

        while let Some(row) = rows.next()? {
            let raw_start: String = row.get(0)?;
            let raw_stop: String = row.get(1)?;
            let week_day: String = row.get(2)?;

            let start: NaiveDateTime = parse_from_str(&raw_start, DATE_FORMAT).expect("Parsing error!");
            let stop: NaiveDateTime = parse_from_str(&raw_stop, DATE_FORMAT).expect("Parsing error!");

            // Look up week day in HashMap and update value. If it doesn't exist insert 0 and then increment.
            let count = week_hours.entry(week_day).or_insert(0.0);
            *count += stop.signed_duration_since(start).num_minutes() as f64 / 60.0;

            for hour in week_hours.values() {
                cells.push(Cell::new(&hour.to_string()));
            }
        }
        table.add_row(Row::new(cells));
    }
        table.printstd();

    Ok(())
}