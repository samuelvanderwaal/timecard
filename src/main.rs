use chrono::{Datelike, Local};
use clap::{App, Arg};
use diesel::sqlite::SqliteConnection;

use timecard::{establish_connection, entries_this_week, write_entry, Entry, NewEntry};

fn main() {
    let conn = establish_connection();

    let matches = App::new("timecard")
        .version("0.1.0")
        .author("Samuel Vanderwaal")
        .about("Time tracking command line program")
        .arg(
            Arg::with_name("entry")
                .short("e")
                .long("entry")
                .value_names(&["start", "stop", "code", "memo"])
                .help("Add a new time entry.")
                .takes_value(true)
                .value_delimiter(","),
        )
        .arg(Arg::with_name("test").short("t"))
        .get_matches();

    if let Some(values) = matches.values_of("entry") {
        process_new_entry(values.collect(), &conn);
    }

    let _entries = entries_this_week(&conn);
}

fn process_new_entry(values: Vec<&str>, conn: &SqliteConnection) {
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

    let code = values[2].to_owned();
    let memo = values[3].to_owned();

    let new_entry = NewEntry {
        start,
        stop,
        code,
        memo,
    };

    // if !write_entry(&conn, &new_entry) {
    //     println!("Failed to write entry: {:?}", new_entry);
    // }
    match write_entry(conn, &new_entry) {
        Ok(_) => (),
        Err(e) => println!("Error writing entry: {:?}", e),
    }
}

fn parse_entry_time(time_str: String) -> (u32, u32) {
    let time = time_str.parse::<u32>().expect("Failed to parse time!");
    (time / 100, time % 100)
}

fn _create_weekly_report(entries: Vec<Entry>) -> String {
    // Loop through entries, sum time for each project.
    String::from("new")
}
