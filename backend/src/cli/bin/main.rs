#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate indexmap;

use anyhow::Result;
use chrono::{Datelike, Duration, Local, NaiveDateTime};
use chrono::offset::TimeZone;
use sqlx::sqlite::SqlitePool;
use clap::{App, Arg};

use std::collections::{HashMap, HashSet};
use indexmap::IndexMap;
use std::str;

use prettytable::{Attr, color, Cell, Row, Table};

use timecard::db::{self, Entry, Project};

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

static DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const MAX_WIDTH: usize = 20;

struct HourRowData {
    project: String,
    hours: IndexMap<String, f64>,
}

struct MemoRowData {
    project: String,
    memos: IndexMap<String, String>,
}

impl HourRowData {
    fn new() -> Self {
        HourRowData {
            project: String::new(),
            hours: indexmap!{
                "Sun".to_string() => 0.0,
                "Mon".to_string() => 0.0,
                "Tue".to_string() => 0.0,
                "Wed".to_string() => 0.0,
                "Thu".to_string() => 0.0,
                "Fri".to_string() => 0.0,
                "Sat".to_string() => 0.0,
            }
        }
    }

    fn convert_to_row(&self, text_color: color::Color) -> Row {
        let mut cells: Vec<Cell> = Vec::new();
        cells.push(
            Cell::new(&self.project)
                .with_style(Attr::ForegroundColor(text_color))
        );
        for (_, value) in self.hours.iter() {
            cells.push(
                Cell::new(&value.to_string())
                    .with_style(Attr::ForegroundColor(text_color))
            );
        }
        Row::new(cells)
    }
}

impl MemoRowData {
    fn new() -> Self {
        MemoRowData {
            project: String::new(),
            memos: indexmap!{
                "Sun".to_string() => String::from(""),
                "Mon".to_string() => String::from(""),
                "Tue".to_string() => String::from(""),
                "Wed".to_string() => String::from(""),
                "Thu".to_string() => String::from(""),
                "Fri".to_string() => String::from(""),
                "Sat".to_string() => String::from(""),
            },
        }
    }

    fn convert_to_row(&self, text_color: color::Color) -> Row {
        let mut cells: Vec<Cell> = Vec::new();
        cells.push(
            Cell::new(&self.project)
                .with_style(Attr::ForegroundColor(text_color))
        );
        for (_, value) in self.memos.iter() {
            cells.push(
                Cell::new(&value)
                    .with_style(Attr::ForegroundColor(text_color))
            );
        }
        Row::new(cells)
    }
}


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
            Arg::with_name("delete_last_entry")
                .short('d')
                .long("delete")
                .about("Delete the most recent entry.")

        )
        .arg(
            Arg::with_name("add_project")
                .short('a')
                .long("add-project")
                .value_names(&["name", "code"])
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
                .takes_value(true)
                .value_name("code")
                .about("Delete a project from the reference table.")
        )
        .get_matches();

    if let Some(values) = matches.values_of("entry") {
        match process_new_entry(values.collect(), &pool).await {
            Ok(_) => println!("Entry submitted."),
            // TODO: Log error
            Err(_e) => eprintln!("Error writing entry."),
        }
        std::process::exit(1);
    }

    if let Some(values) = matches.values_of("backdate") {
        match backdated_entry(values.collect(), &pool).await {
            Ok(_) => println!("Entry submitted."),
            // TODO: Log error
            Err(_e) => println!("Error writing entry."),
        }
        std::process::exit(1);
    }

    if let Some(value) = matches.value_of("week") {
        let mut _memos = false;
        let _num = match value.parse::<i64>() {
            Ok(n) => n,
            // TODO: Log error
            Err(_e) => {
                eprintln!("Error: week value must be an integer.");
                std::process::exit(1);
            }
        };

        if matches.is_present("with_memos") {
            _memos = true;
        }

        create_weekly_report(&pool, _num, _memos).await?;
        std::process::exit(1);
    }

    if matches.is_present("last_entry") {
        match display_last_entry(&pool).await {
            Ok(table) => table.printstd(),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        };
        std::process::exit(1);
    }

    if matches.is_present("delete_last_entry") {
        match db::delete_last_entry(&pool).await {
            Ok(_) => println!("Most recent entry deleted."),
            Err(e) => println!("Error: {:?}", e),
        }
    }

    if let Some(values) = matches.values_of("add_project") {
        let values: Vec<&str> = values.collect();
        let new_project = Project{
            id: None,
            name: values[0].to_string(),
            code: values[1].to_string(),
        };

        let id = db::write_project(&pool, &new_project).await?;

        println!("Project written with id: {}", id);
    }

    if matches.is_present("list_projects") {
        let projects = db::read_all_projects(&pool).await?;

        for project in projects {
            println!("Name: {}\nCode: {}\n", project.name, project.code);
        }
    }

    if let Some(value) = matches.value_of("delete_project") {
        let code = value.parse::<String>()?;

        db::delete_project(&pool, code).await?;

        println!("Project deleted.");
    }


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
            let date_values: Vec<&str> = values[0].split('-').collect();
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

async fn create_weekly_report(pool: &SqlitePool, num_weeks: i64, with_memos: bool) -> Result<()> {
    let parse_from_str = NaiveDateTime::parse_from_str;
    
    let day_of_week: String = Local::today().weekday().to_string();
    let offset = *WEEKDAYS.get(&day_of_week).expect("Day does not exist!") + (7 * num_weeks);
    let week_beginning = Local::today() - Duration::days(offset);
    let week_ending = week_beginning + Duration::days(6);

    let entries = db::read_entries_between(pool, week_beginning.to_string(), week_ending.to_string()).await?;

    let mut table = Table::new();
    table.add_row(row![Fb => "Project", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]);

    let mut codes: HashSet<String> = HashSet::new();
    for entry in &entries {
        codes.insert(entry.code.clone());
    } 

    for (index, code) in codes.iter().enumerate() {
        let mut hour_data = HourRowData::new();
        let mut memo_data = MemoRowData::new();
        hour_data.project = code.clone();
        memo_data.project = code.clone();

        let project_entries = entries.iter().filter(|entry| &entry.code == code);

        for entry in project_entries {
                let start: NaiveDateTime =
                    parse_from_str(&entry.start, DATE_FORMAT).expect("Parsing error!");
                let stop: NaiveDateTime =
                    parse_from_str(&entry.stop, DATE_FORMAT).expect("Parsing error!");
                let h = hour_data.hours.entry(entry.week_day.clone()).or_insert(0.0);
                *h += stop.signed_duration_since(start).num_minutes() as f64 / 60.0;
                
                let current_memo = memo_data.memos.entry(entry.week_day.clone()).or_insert(String::from(""));
                // Implement max width
                for chunk in entry.memo.as_bytes().chunks(MAX_WIDTH) {
                    let chunk_str = str::from_utf8(chunk)?;
                    (*current_memo).push_str(chunk_str);
                    if chunk_str.len() >= MAX_WIDTH {
                        (*current_memo).push_str("\n");
                    }
                }
                (*current_memo).push_str("; ");
                (*current_memo).push_str("\n");
        }

        let text_color = if index % 2 == 1 { color::MAGENTA } else { color::WHITE };
        
        table.add_row(hour_data.convert_to_row(text_color));

        if with_memos {
            table.add_row(memo_data.convert_to_row(text_color));
        }
    }
    table.printstd();

    Ok(())
}

async fn display_last_entry(pool: &SqlitePool) -> Result<Table> {
    let e = db::read_last_entry(&pool).await?;

    let mut table = Table::new();
    table.add_row(row![Fb => "Start Time", "Stop Time", "Week Day", "Code", "Memo"]);
    table.add_row(row![e.start, e.stop, e.week_day, e.code, e.memo]);

    Ok(table)
}