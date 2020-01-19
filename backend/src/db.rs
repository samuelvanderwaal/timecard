use std::collections::HashMap;
use rusqlite::{params, Connection, Result as SqlResult, NO_PARAMS};
use chrono::{Datelike, Duration, Local};

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

#[derive(Debug, Clone)]
pub struct ProjectEntries {
    pub code: String,
    pub entries: Vec<Entry>,
}

lazy_static! {
    pub static ref WEEKDAYS: HashMap<String, i64> = vec![
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

pub static DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub fn establish_connection() -> Connection {
    dotenv().ok();
    let db_url = env::var("TIMECARD_DB").expect("Database url must be set!");
    let conn = Connection::open(db_url).expect("Could not open connection!");

    // Create tables if they don't already exist.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS entries (
        id INTEGER PRIMARY KEY,
        start TEXT NOT NULL,
        stop TEXT NOT NULL,
        week_day TEXT NOT NULL,
        code TEXT NOT NULL,
        memo TEXT NOT NULL
        )",
        NO_PARAMS,
    )
    .expect("Connection execution error!");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        code TEXT NOT NULL
        )",
        NO_PARAMS,
    )
    .expect("Connection execution error!");
    conn
}

pub fn write_entry(conn: &Connection, new_entry: &NewEntry) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO entries (start, stop, week_day, code, memo)
            VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            new_entry.start,
            new_entry.stop,
            new_entry.week_day,
            new_entry.code,
            new_entry.memo
        ],
    )?;
    Ok(())
}

pub fn query_all_projects(conn: &Connection) -> SqlResult<Vec<Project>> {
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

pub fn query_project(conn: &Connection, code: &str) -> SqlResult<Project> {
    let query = format!("SELECT * FROM projects where code={}", code);
    let mut stmt = conn.prepare(&query)?;
    let mut project_iter = stmt.query_map(params![], |row| {
        Ok(Project {
            id: row.get(0)?,
            name: row.get(1)?,
            code: row.get(2)?,
        })
    })?;

    Ok(match project_iter.next() {
        Some(project) => project?,
        None => panic!("No such project!")
    })
}

pub fn query_weekly_entries(conn: &Connection, weeks_ago: i64) -> SqlResult<Vec<ProjectEntries>> {
    let projects = query_all_projects(conn)?;
    let day_of_week: String = Local::today().weekday().to_string();

    // Offset is number required to go to beginning of week + 7 * num to find number of weeks we go back.
    let offset = *WEEKDAYS.get(&day_of_week).expect("Day does not exist!") + (7 * weeks_ago);
    let week_beginning = Local::today() - Duration::days(offset);
    let week_ending = week_beginning + Duration::days(7);

    // let parse_from_str = NaiveDateTime::parse_from_str;

    // Remove tz offsets.
    let week_beginning = week_beginning.naive_local();
    let week_ending = week_ending.naive_local();

    let mut project_entries: Vec<ProjectEntries> = Vec::new();

    for project in projects {
        let mut entries: Vec<Entry> = Vec::new();
        let query = format!(
            "SELECT id, start, stop, week_day, memo FROM entries WHERE code='{}' AND start > '{}' and start < '{}';",
            project.code, week_beginning, week_ending
            );
        let mut stmt = conn.prepare(&query)?;
        let entries_iter = stmt.query_map(NO_PARAMS, |row| {
            Ok(Entry {
                id: row.get(0)?,
                start: row.get(1)?,
                stop: row.get(2)?,
                week_day: row.get(3)?,
                code: project.code.clone(),
                memo: row.get(4)?,
            })
        })?;

        for entry in entries_iter {
            entries.push(entry?);
        }

        project_entries.push(ProjectEntries{
            code: project.code,
            entries: entries,
        });
    }
    Ok(project_entries)
}

pub fn query_last_entry(conn: &Connection) -> SqlResult<Entry> {
    let query = "SELECT * FROM entries ORDER BY id DESC LIMIT 1";
    let mut stmt = conn.prepare(&query)?;
    let mut entries_iter = stmt.query_map(NO_PARAMS, |row| {
        Ok(Entry {
            id: row.get(0)?,
            start: row.get(1)?,
            stop: row.get(2)?,
            week_day: row.get(3)?,
            code: row.get(4)?,
            memo: row.get(5)?,
        })
    })?;

    // Our query ensures only one result so we can safely assume our entry is the first and only value in the Iter.
    match entries_iter.next() {
        Some(e) => e,
        None => panic!("No entry found!"),  
    }
}

pub fn delete_last_entry(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM entries WHERE id = (SELECT MAX(id) FROM entries LIMIT 1);",
        params![],
    )?;
    Ok(())
}

pub fn insert_project(conn: &Connection, name: String, code: String) -> SqlResult<()> {
    let stmt = format!(
        "INSERT INTO projects(name, code) VALUES('{}', '{}');",
        name.trim_end(),
        code.trim_end()
    );
    conn.execute(&stmt, params![])?;

    Ok(())
}

pub fn delete_project(conn: &Connection, code: String) -> SqlResult<()> {
    let stmt = format!("DELETE FROM projects WHERE code='{}';", code);
    conn.execute(&stmt, params![])?;
    Ok(())
}