use anyhow::Result;
use warp::Filter;
use sqlx::sqlite::SqlitePool;

use timecard::db;
use timecard::api;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = db::setup_pool().await?;
    db::setup_db(&pool).await?;
    run(pool).await;

    Ok(())
}

async fn run(pool: SqlitePool) {
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

    warp::serve(routes).run(([0, 0, 0, 0], 3333)).await;
}