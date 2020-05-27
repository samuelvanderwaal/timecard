#![allow(dead_code)]
use anyhow::Result;

use warp::Filter;

use sqlx::sqlite::SqlitePool;

use timecard::db;
use timecard::api;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = db::setup_pool().await?;
    run(pool).await;

    Ok(())
}

async fn run(pool: SqlitePool) {
    let routes = api::post_entry(pool.clone())
                    .or(api::get_entry(pool.clone()))
                    .or(api::update_entry(pool));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}