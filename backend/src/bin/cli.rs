#[macro_use]
extern crate clap;
#[macro_use]
extern crate prettytable;

use std::str::{self, Utf8Error};
use std::io::{stdin, stdout, Write};
use chrono::{Datelike, Local, NaiveDateTime, Duration};
use clap::{App, Arg};
use rusqlite::{Connection, Result as SqlResult};
use indexmap::IndexMap;
use prettytable::{Table, Row, Cell};

use backend::db::*;

const MAX_WIDTH: usize = 20;

fn main() {
    let conn = establish_connection();

    let matches = App::new("timecard")
        .version(crate_version!())
        .author("Samuel Vanderwaal")
        .about("A time-tracking command line program.")
        .arg(
            Arg::with_name("entry")
                .short("e")
                .long("entry")
                .value_names(&["start", "stop", "code", "memo"])
                .help("Add a new time entry.")
                .takes_value(true)
                .value_delimiter(","),
        )
        .arg(
            Arg::with_name("week")
                .short("w")
                .long("week")
                .takes_value(true)
                .help("Print weekly report."),
        )
        .arg(
            Arg::with_name("with_memos")
                .short("m")
                .long("with-memos")
                .help("Use with '-w`. Adds memos to weekly report.")
        )
        .arg(
            Arg::with_name("last_entry")
                .short("l")
                .long("last")
                .help("Display most recent entry."),
        )
        .arg(
            Arg::with_name("delete_entry")
                .short("d")
                .long("delete")
                .help("Delete the most recent entry."),
        )
        .arg(
            Arg::with_name("add_project")
                .short("a")
                .long("add-project")
                .help("Add a new project to the reference table."),
        )
        .arg(
            Arg::with_name("list_projects")
                .short("p")
                .long("list-projects")
                .help("List all projects in the reference table."),
        )
        .arg(
            Arg::with_name("delete_project")
                .long("delete-project")
                .help("Delete project from the reference table."),
        )
        .get_matches();

    if let Some(values) = matches.values_of("entry") {
        process_new_entry(values.collect(), &conn);
    }

    if let Some(value) = matches.value_of("week") {
        let mut memos = false;
        let num = match value.parse::<i64>() {
            Ok(n) => n,
            Err(_) => {
                println!("Error: value must be an integer.");
                std::process::exit(1);
            }
        };
        if matches.is_present("with_memos") {
            memos = true;
        }
        match create_weekly_report(&conn, num, memos) {
            Ok(()) => (),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    if matches.is_present("last_entry") {
        match display_last_entry(&conn) {
            Ok(()) => (),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    if matches.is_present("delete_entry") {
        match delete_last_entry(&conn) {
            Ok(()) => println!("Most recent entry deleted."),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    if matches.is_present("add_project") {
        match add_new_project(&conn) {
            Ok(()) => println!("Project added."),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    if matches.is_present("list_projects") {
        match list_projects(&conn) {
            Ok(()) => (),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    if matches.is_present("delete_project") {
        match remove_project(&conn) {
            Ok(()) => (),
            Err(e) => println!("Error: {:?}", e),
        }
    }
}

fn process_new_entry(values: Vec<&str>, conn: &Connection) {
    let now = Local::now();
    let year = now.year();
    let month = now.month();
    let day = now.day();

    let (start_hour, start_minute) = parse_entry_time(values[0].to_owned());
    let (stop_hour, stop_minute) = parse_entry_time(values[1].to_owned());

    let start = format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, start_hour, start_minute, 0
    );
    let stop = format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, stop_hour, stop_minute, 0
    );

    let week_day: String = Local::today().weekday().to_string();
    let code = values[2].to_owned();
    let memo = values[3].to_owned();

    let new_entry = NewEntry {
        start,
        stop,
        week_day,
        code,
        memo,
    };

    match write_entry(conn, &new_entry) {
        Ok(_) => println!("Entry submitted."),
        Err(e) => println!("Error writing entry: {:?}", e),
    }
}

fn parse_entry_time(time_str: String) -> (u32, u32) {
    let time = time_str.parse::<u32>().expect("Failed to parse time!");
    (time / 100, time % 100)
}

fn format_memo(entry_memo: String, char_width: usize) -> Result<String, Utf8Error> {
    let mut formatted_memo = String::new();
    for chunk in entry_memo.as_bytes().chunks(char_width) {
        let chunk_str = str::from_utf8(chunk)?;
        (formatted_memo).push_str(chunk_str);
        if !(chunk_str.len() < char_width) {
            (formatted_memo).push_str("\n");
        }
    }
    (formatted_memo).push_str("; ");
    (formatted_memo).push_str("\n");
    Ok(formatted_memo)
}

fn create_weekly_report(conn: &Connection, weeks_ago: i64, with_memos: bool) -> SqlResult<()> {
    let project_entries = query_weekly_entries(conn, weeks_ago)?;
    let day_of_week: String = Local::today().weekday().to_string();

    // Offset is number required to go to beginning of week + 7 * num to find number of weeks we go back.
    let offset = *WEEKDAYS.get(&day_of_week).expect("Day does not exist!") + (7 * weeks_ago);
    let week_beginning = Local::today() - Duration::days(offset);
    println!("Week Beginning: {:?}", week_beginning);

    let parse_from_str = NaiveDateTime::parse_from_str;

    // Set up table for printing.
    let mut table = Table::new();
    table.add_row(row![Fb => "Project", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);

    for project in project_entries {
        let mut all_zeros = true;
        let mut no_memos = true;

        // Set up hashmap to track hours per week day.
        let mut week_hours: IndexMap<String, f64> = IndexMap::new();
        let mut week_memos: IndexMap<String, String> = IndexMap::new();

        let week_days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        for day in week_days.iter() {
            week_hours.insert((*day).to_owned(), 0.0);
            week_memos.insert((*day).to_owned(), String::from(""));
        }

        // Set up rows and add project code.
        let mut time_cells: Vec<Cell> = Vec::new();
        let mut memo_cells: Vec<Cell> = Vec::new();
        time_cells.push(Cell::new(&project.code));
        if with_memos{
            memo_cells.push(Cell::new(" "));
        }

        for entry in project.entries {
            let start: NaiveDateTime =
                parse_from_str(&entry.start, DATE_FORMAT).expect("Parsing error!");
            let stop: NaiveDateTime =
                parse_from_str(&entry.stop, DATE_FORMAT).expect("Parsing error!");

            // Look up week day in the IndexMap and update value. If it doesn't exist insert 0 and then increment.
            let count = week_hours.entry(entry.week_day.clone()).or_insert(0.0);
            *count += stop.signed_duration_since(start).num_minutes() as f64 / 60.0;
            
            // Look up the week day memos IndexMap and concatenate memos.
            let daily_memo = week_memos.entry(entry.week_day).or_insert(String::from(""));

            let formatted_memo = format_memo(entry.memo, MAX_WIDTH)?;
            (*daily_memo).push_str(&formatted_memo);
        }
        // Iterate over hashmap hour values and add to cells.
        for hour in week_hours.values() {
            if *hour > 0.0 {
                all_zeros = false;
            }
            time_cells.push(Cell::new(&hour.to_string()));
        }

        for memo in week_memos.values() {
            if ! (*memo).is_empty() {
                no_memos = false;
            } 
            memo_cells.push(Cell::new(&memo.to_string()));
        }

        // Only add rows with at least one non-zero value.
        if !all_zeros {
            table.add_row(Row::new(time_cells.clone()));
        }

        if !no_memos && with_memos {
            table.add_row(Row::new(memo_cells.clone()));
        }

    }
    table.printstd();
    Ok(())
}

fn display_last_entry(conn: &Connection) -> SqlResult<()> {
    let mut table = Table::new();
    table.add_row(row![Fb => "Start Time", "Stop Time", "Week Day", "Code", "Memo"]);
    let mut cells: Vec<Cell> = Vec::new();

    let entry = query_last_entry(conn)?;

    let formatted_memo = format_memo(entry.memo, MAX_WIDTH)?;

    cells.push(Cell::new(&entry.start));
    cells.push(Cell::new(&entry.stop));
    cells.push(Cell::new(&entry.week_day));
    cells.push(Cell::new(&entry.code));
    cells.push(Cell::new(&formatted_memo));

    table.add_row(Row::new(cells));

    table.printstd();

    Ok(())
}

pub fn add_new_project(conn: &Connection) -> SqlResult<()> {
    let mut name = String::new();
    let mut code = String::new();

    print!("Project name: ");
    // Std out is line-buffered by default so flush to print output immediately.
    stdout().flush().unwrap();
    stdin()
        .read_line(&mut name)
        .expect("Failed to read from std in!");

    print!("Project code (e.g.: 19-165): ");
    stdout().flush().unwrap();
    stdin()
        .read_line(&mut code)
        .expect("Failed to read from std in!");

    name = name.trim_end().to_string();
    code = code.trim_end().to_string();

    insert_project(conn, name, code)?;

    Ok(())
}

pub fn list_projects(conn: &Connection) -> SqlResult<()> {
    let projects: Vec<Project> = query_all_projects(&conn)?;

    let mut table = Table::new();
    table.add_row(row![Fb => "Project Name", "Project Code"]);

    for project in projects {
        table.add_row(row![project.name, project.code]);
    }
    table.printstd();

    Ok(())
}

pub fn remove_project(conn: &Connection) -> SqlResult<()> {
    let mut code = String::new();
    let mut confirmation = String::new();

    print!("Project code to DELETE: ");
    stdout().flush().unwrap();
    stdin()
        .read_line(&mut code)
        .expect("Failed to read from std in!");

    code = code.trim_end().to_string();

    print!("Are you sure you want to delete project: {}? Y/N  ", code);
    stdout().flush().unwrap();
    stdin()
        .read_line(&mut confirmation)
        .expect("Failed to read from std in!");

    confirmation = confirmation.trim_end().to_lowercase();

    if confirmation == "y" {
        // Check that project exists.
        query_project(conn, &code)?;

        delete_project(conn, code)?;
        println!("Project deleted.");
    } else {
        println!("Canceled!");
    }
    Ok(())
}
