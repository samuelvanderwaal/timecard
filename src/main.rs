use clap::{Arg, App};

use timecard::{establish_connection, NewEntry, write_entry};

fn main() {
    let conn = establish_connection();

    let matches = App::new("timecard")
                    .version("0.1.0")
                    .author("Samuel Vanderwaal")
                    .about("Time tracking command line program")
                    .arg(Arg::with_name("entry")
                        .short("e")
                        .long("entry")
                        .value_names(&["start", "stop", "code", "memo"])
                        .help("Add a new time entry.")
                        .takes_value(true)
                        .value_delimiter(","))
                    .arg(Arg::with_name("test")
                        .short("t"))
                    .get_matches();

    if let Some(values) = matches.values_of("entry")  {
        let values: Vec<&str> = values.collect();
        let start = values[0].to_owned();
        let stop = values[1].to_owned();
        let code = values[2].to_owned();
        let memo = values[3].to_owned();
        let new_entry = NewEntry { start, stop, code, memo };

        // if !write_entry(&conn, &new_entry) {
        //     println!("Failed to write entry: {:?}", new_entry);
        // }
        match write_entry(&conn, &new_entry) {
            Ok(_) => (),
            Err(e) => println!("Error writing entry: {:?}", e),
        }
    }
}