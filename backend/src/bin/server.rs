use r2d2;
use r2d2_sqlite::SqliteConnectionManager;
use warp::{Filter};

use dotenv::dotenv;
use std::env;

use timecard::db::{write_entry, NewEntry};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("TIMECARD_DB").expect("Database url must be set!");    
    let manager = SqliteConnectionManager::file(db_url);
    let pool = r2d2::Pool::new(manager).unwrap();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        // .allow_origin("http://localhost:8080")
        .allow_methods(vec!["GET", "POST", "DELETE"]);
    
    // let new_entry = warp::path!("new_entry")
    // let last_entry = warp::path("last_entry").map(|| get_last_entry()).with(cors);
    let new_entry = warp::post()
        .and(warp::path!("new_entry"))
        .and(warp::body::json())
        .map(move |entry: NewEntry| {
            let conn = pool.get().unwrap();
            write_entry(&conn, &entry).unwrap(); 
            warp::reply()
        })
        .with(cors);

    warp::serve(new_entry).run(([127, 0, 0, 1], 3030)).await;
}