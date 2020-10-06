// Crates
use anyhow::Result;
use dotenv::dotenv;
use sqlx::sqlite::SqlitePool;
use tracing::{info};
use warp::Filter;

// Local
use timecard::api;
use timecard::db;
use timecard::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let listen_port = 3333;
    let pool = db::setup_pool().await?;
    db::setup_db(&pool).await?;

    let subscriber = get_subscriber("timecard".into(), "info".into());
    init_subscriber(subscriber);

    info!("Listening on port {}. . .", listen_port);
    run(pool, listen_port).await;

    Ok(())
}

async fn run(pool: SqlitePool, listen_port: u16) {
    let routes = api::post_entry(pool.clone())
        .or(api::get_entry(pool.clone()))
        .or(api::update_entry(pool.clone()))
        .or(api::get_entries_between(pool.clone()))
        .or(api::read_last_entry(pool.clone()))
        .or(api::delete_entry(pool.clone()))
        .or(api::delete_last_entry(pool.clone()))
        .or(api::post_project(pool.clone()))
        .or(api::get_project(pool.clone()))
        .or(api::get_all_projects(pool.clone()))
        .or(api::update_project(pool.clone()))
        .or(api::delete_project(pool.clone()));

    warp::serve(routes).run(([0, 0, 0, 0], listen_port)).await;
}
