#[macro_use]
extern crate clap;

use chrono::{Datelike, Local};
use clap::{App, Arg};
use rusqlite::{Connection, Result};
use timecard::*;

fn main() -> Result<()> {
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
        let num = match value.parse::<i64>() {
            Ok(n) => n,
            Err(_) => {
                println!("Error: value must be an integer.");
                std::process::exit(1);
            }
        };
        match create_weekly_report(&conn, num) {
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
        match delete_project(&conn) {
            Ok(()) => (),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    Ok(())
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
