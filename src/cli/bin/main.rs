// Macros
#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prettytable;
#[macro_use]
extern crate indexmap;
#[macro_use]
extern crate anyhow;

// Std
use std::collections::{HashMap, HashSet};
use std::env;
use std::str;

// Crates
use anyhow::{Context, Result};
use chrono::offset::TimeZone;
use chrono::{Datelike, Duration, Local, NaiveDateTime};
use clap::{App, Arg};
use dotenv::dotenv;
use http::StatusCode;
use indexmap::IndexMap;
use prettytable::{color, Attr, Cell, Row, Table};
use reqwest::Client;

// Local
use timecard::{Entry, Project};

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
            hours: indexmap! {
                "Sun".to_string() => 0.0,
                "Mon".to_string() => 0.0,
                "Tue".to_string() => 0.0,
                "Wed".to_string() => 0.0,
                "Thu".to_string() => 0.0,
                "Fri".to_string() => 0.0,
                "Sat".to_string() => 0.0,
            },
        }
    }

    fn convert_to_row(&self, text_color: color::Color) -> Row {
        let mut cells: Vec<Cell> = Vec::new();
        cells.push(Cell::new(&self.project).with_style(Attr::ForegroundColor(text_color)));
        for (_, value) in self.hours.iter() {
            cells.push(Cell::new(&value.to_string()).with_style(Attr::ForegroundColor(text_color)));
        }
        Row::new(cells)
    }
}

impl MemoRowData {
    fn new() -> Self {
        MemoRowData {
            project: String::new(),
            memos: indexmap! {
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
        cells.push(Cell::new(&self.project).with_style(Attr::ForegroundColor(text_color)));
        for (_, value) in self.memos.iter() {
            cells.push(Cell::new(&value).with_style(Attr::ForegroundColor(text_color)));
        }
        Row::new(cells)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let base_url: String = env::var("BASE_URL").context("BASE_URL env var must be set!")?;

    let client = Client::new();

    let matches = App::new("timecard")
        .version(crate_version!())
        .author("Samuel Vanderwaal")
        .help("A time-tracking command line program.")
        .arg(
            Arg::with_name("entry")
                .short("e")
                .long("entry")
                .value_names(&["start", "stop", "code", "memo"])
                .help("Add a new time entry.")
                .takes_value(true)
                .value_delimiter("|"),
        )
        .arg(
            Arg::with_name("backdate")
                .short("b")
                .long("backdate")
                .value_names(&["backdate", "start", "stop", "code", "memo"])
                .help("Add a backdated entry.")
                .takes_value(true)
                .value_delimiter("|"),
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
                .help("Use with '-w'. Adds memos to weekly report."),
        )
        .arg(
            Arg::with_name("last_entry")
                .long("last")
                .help("Display the most recent entry."),
        )
        .arg(
            Arg::with_name("delete_last_entry")
                .short("d")
                .long("delete")
                .help("Delete the most recent entry."),
        )
        .arg(
            Arg::with_name("add_project")
                .short("a")
                .long("add-project")
                .value_names(&["name", "code"])
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
                .takes_value(true)
                .value_name("code")
                .help("Delete a project from the reference table."),
        )
        .get_matches();

    if let Some(values) = matches.values_of("entry") {
        match process_new_entry(&base_url, client, values.collect()).await {
            Ok(_) => println!("Entry submitted."),
            // TODO: Log error
            Err(e) => eprintln!("Error writing entry: {}", e),
        }
        std::process::exit(1);
    }

    if let Some(values) = matches.values_of("backdate") {
        match backdated_entry(&base_url, client, values.collect()).await {
            Ok(_) => println!("Entry submitted."),
            // TODO: Log error
            Err(_e) => println!("Error writing entry."),
        }
        std::process::exit(1);
    }

    if let Some(value) = matches.value_of("week") {
        let mut memos = false;
        let num = match value.parse::<i64>() {
            Ok(n) => n,
            // TODO: Log error
            Err(_e) => {
                eprintln!("Error: week value must be an integer.");
                std::process::exit(1);
            }
        };

        if matches.is_present("with_memos") {
            memos = true;
        }

        create_weekly_report(&base_url, client, num, memos).await?;
        std::process::exit(1);
    }

    if matches.is_present("last_entry") {
        match display_last_entry(&base_url, client).await {
            Ok(table) => table.printstd(),
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        };
        std::process::exit(1);
    }

    if matches.is_present("delete_last_entry") {
        let url = format!("{}/delete_last_entry", &base_url);
        let res = client.post(&url).send().await?;

        match res.status() {
            StatusCode::OK => println!("Most recent entry deleted."),
            _ => println!("Error: {:?}", res.status()),
        }
    }

    if let Some(values) = matches.values_of("add_project") {
        let values: Vec<&str> = values.collect();
        let new_project = Project {
            id: None,
            name: values[0].to_string(),
            code: values[1].to_string(),
        };

        let url = format!("{}/project", &base_url);
        let res = client.post(&url).json(&new_project).send().await?;

        if res.status().is_success() {
            println!("Project saved.");
        } else {
            println!("Http error: {}", res.status());
        }
    }

    if matches.is_present("list_projects") {
        let url = format!("{}/all_projects", &base_url);
        let projects = client
            .get(&url)
            .send()
            .await?
            .json::<Vec<Project>>()
            .await?;

        let mut table = Table::new();
        table.add_row(row![Fb => "Name", "Code"]);

        for project in projects {
            table.add_row(row![project.name, project.code]);
        }
        table.printstd();
    }

    if let Some(value) = matches.value_of("delete_project") {
        let code = value.parse::<String>()?;

        let url = format!("{}/delete_project/{}", &base_url, code);
        let res = client.post(&url).send().await?;

        if res.status().is_success() {
            println!("Project deleted.");
        } else {
            println!("Http error: {}", res.status());
        }
    }

    Ok(())
}

async fn process_new_entry(base_url: &str, client: Client, values: Vec<&str>) -> Result<()> {
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

    let url = format!("{}/entry", base_url);
    let res = client.post(&url).json(&new_entry).send().await?;

    match res.status() {
        StatusCode::OK => Ok(()),
        _ => Err(anyhow!("Status code: {}", res.status())),
    }
}

async fn backdated_entry(base_url: &str, client: Client, values: Vec<&str>) -> Result<()> {
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
        }
    };

    let (start_hour, start_minute) = parse_entry_time(values[1].to_owned())?;
    let (stop_hour, stop_minute) = parse_entry_time(values[2].to_owned())?;

    let start = entry_time_to_full_date(date, start_hour, start_minute);
    let stop = entry_time_to_full_date(date, stop_hour, stop_minute);

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

    let url = format!("{}/entry", base_url);
    let res = client.post(&url).json(&new_entry).send().await?;

    match res.status() {
        StatusCode::OK => Ok(()),
        _ => Err(anyhow!("Status code: {}", res.status())),
    }
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
    );
}

async fn create_weekly_report(
    base_url: &str,
    client: Client,
    num_weeks: i64,
    with_memos: bool,
) -> Result<()> {
    let parse_from_str = NaiveDateTime::parse_from_str;

    let day_of_week: String = Local::today().weekday().to_string();
    let offset = *WEEKDAYS.get(&day_of_week).expect("Day does not exist!") + (7 * num_weeks);
    let week_beginning = Local::today() - Duration::days(offset);
    let week_ending = week_beginning + Duration::days(6);

    let url = format!(
        "{}/entries_between/{}/{}",
        base_url, week_beginning, week_ending
    );
    let entries = client.get(&url).send().await?.json::<Vec<Entry>>().await?;

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

            let current_memo = memo_data
                .memos
                .entry(entry.week_day.clone())
                .or_insert(String::from(""));
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

        let text_color = if index % 2 == 1 {
            color::MAGENTA
        } else {
            color::WHITE
        };

        table.add_row(hour_data.convert_to_row(text_color));

        if with_memos {
            table.add_row(memo_data.convert_to_row(text_color));
        }
    }
    table.printstd();

    Ok(())
}

async fn display_last_entry(base_url: &str, client: Client) -> Result<Table> {
    let url = format!("{}/last_entry", base_url);
    let e = client.get(&url).send().await?.json::<Entry>().await?;

    let mut table = Table::new();
    table.add_row(row![Fb => "Start Time", "Stop Time", "Week Day", "Code", "Memo"]);
    table.add_row(row![e.start, e.stop, e.week_day, e.code, e.memo]);

    Ok(table)
}
