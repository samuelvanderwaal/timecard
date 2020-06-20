#[macro_use]
extern crate clap;

use anyhow::Result;
use clap::{App, Arg};
use chrono::{Datelike, DateTime, Duration, Local};
use chrono::offset::TimeZone;
use sqlx::sqlite::SqlitePool;

use timecard::db::{self, Entry};

#[tokio::main]
async fn main() -> Result<()>{
    let pool = db::setup_pool().await?;

    let matches = App::new("timecard")
        .version(crate_version!())
        .author("Samuel Vanderwaal")
        .about("A time-tracking command line program.")
        .arg(
            Arg::with_name("entry")
                .short('e')
                .long("entry")
                .value_names(&["start", "stop", "code", "memo"])
                .about("Add a new time entry.")
                .takes_value(true)
                .value_delimiter(" ")
        )
        .arg(
            Arg::with_name("backdate")
                .short('b')
                .long("backdate")
                .value_names(&["backdate", "start", "stop", "code", "memo"])
                .about("Add a backdated entry.")
                .takes_value(true)
                .value_delimiter(" ")
        )
        .arg(
            Arg::with_name("week")
                .short('w')
                .long("week")
                .takes_value(true)
                .about("Print weekly report.")
        )
        .arg(
            Arg::with_name("with_memos")
            .short('m')
            .long("with-memos")
            .about("Use with '-w'. Adds memos to weekly report.")
        )
        .arg(
            Arg::with_name("last_entry")
                .long("last")
                .about("Display the most recent entry.")
        )
        .arg(
            Arg::with_name("delete_entry")
                .short('d')
                .long("delete")
                .about("Delete the most recent entry.")

        )
        .arg(
            Arg::with_name("add_project")
                .short('a')
                .long("add-project")
                .about("Add a new project to the reference table.")
        )
        .arg(
            Arg::with_name("list_projects")
                .short('p')
                .long("list-projects")
                .about("List all projects in the reference table.")
        )
        .arg(
            Arg::with_name("delete_project")
                .long("delete-project")
                .about("Delete a project from the reference table.")
        )
        .get_matches();

    if let Some(values) = matches.values_of("entry") {
        match process_new_entry(values.collect(), &pool).await {
            Ok(_) => println!("Entry submitted."),
            // TODO: Log error
            Err(_e) => eprintln!("Error writing entry."),
        }
    }

    if let Some(values) = matches.values_of("backdate") {
        match backdated_entry(values.collect(), &pool).await {
            Ok(_) => println!("Entry submitted."),
            // TODO: Log error
            Err(_e) => println!("Error writing entry."),
        }
    }

    // if let Some(value) = matches.value_of("week") {
    //     let mut memos = false;
    //     let num = match value.parse::<i64>() {
    //         Ok(n) => n,
    //         // TODO: Log error
    //         Err(_e) => {
    //             eprintln!("Error: week value must be an integer.");
    //             std::process::exit(1);
    //         }
    //     };

    //     if matches.is_present("with_memos") {
    //         memos = true;
    //     }

    //     match create_weekly_report(&pool)
    // }


    Ok(())
}

async fn process_new_entry(values: Vec<&str>, pool: &SqlitePool) -> Result<()> {
    let (start_hour, start_minute) = parse_entry_time(values[0].to_owned())?;
    let (stop_hour, stop_minute) = parse_entry_time(values[1].to_owned())?;

    let date = Local::now();

    let start = entry_time_to_full_date(date, start_hour, start_minute);
    let stop = entry_time_to_full_date(date, stop_hour, stop_minute);
    let week_day: String = Local::today().weekday().to_string();
    let code = values[2].to_owned();
    let memo = values[3].to_owned();

    let new_entry = Entry {
        id: None,
        start,
        stop,
        week_day,
        code,
        memo,
    };

    db::write_entry(pool, &new_entry).await?;

    Ok(())
}

async fn backdated_entry(values: Vec<&str>, pool: &SqlitePool) -> Result<()> {
    let date = match values[0] {
        "today" => Local::today(),
        "yesterday" => Local::today() - Duration::days(1),
        "tomorrow" => Local::today() + Duration::days(1),
        _ => {
            let date_values: Vec<&str> = values[0].split("-").collect();
            let year: i32 = date_values[0].parse()?;
            let month: u32 = date_values[1].parse()?;
            let day: u32 = date_values[2].parse()?;

            Local.ymd(year, month, day)
            }, 
    };

    let (start_hour, start_minute) = parse_entry_time(values[1].to_owned())?;
    let (stop_hour, stop_minute) = parse_entry_time(values[2].to_owned())?;

    let start = entry_time_to_full_date(date, start_hour, start_minute);
    let stop =  entry_time_to_full_date(date, stop_hour, stop_minute);

    let week_day: String = date.weekday().to_string();
    let code = values[3].to_owned();
    let memo = values[4].to_owned();

    let new_entry = Entry {
        id: None,
        start,
        stop,
        week_day,
        code,
        memo,
    };

    db::write_entry(pool, &new_entry).await?;

    Ok(())
}

fn parse_entry_time(time_str: String) -> Result<(u32, u32)> {
    let time = time_str.parse::<u32>()?;
    Ok((time / 100, time % 100))
}

fn entry_time_to_full_date<T: Datelike>(date: T, hour: u32, minute: u32) -> String {
    let year = date.year();
    let month = date.month();
    let day = date.day();

    return format!(
        "{}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, 0
    )
}