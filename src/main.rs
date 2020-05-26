use std::convert::Infallible;
use warp::{http, Filter};
use warp::reply::Reply;
use anyhow::Result;
use serde_json;

mod db;

use db::Entry;
use sqlx::sqlite::SqlitePool;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = db::setup_pool().await?;
    run(pool).await;

    Ok(())
}

async fn run(pool: SqlitePool) {
    let routes = post_entry(pool.clone()).or(get_entry(pool));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

fn json_body() -> impl Filter<Extract = (Entry,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn with_pool(pool: SqlitePool) -> impl Filter<Extract = (SqlitePool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

// Filters
fn post_entry(pool: SqlitePool) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("entry")
        .and(warp::post())
        .and(json_body())
        .and(with_pool(pool))
        .and_then(new_entry)
}

fn get_entry(pool: SqlitePool) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        // .and(warp::path!("entry" / i32))
        .and(warp::path("entry"))
        .and(warp::path::param::<i32>())
        .and(with_pool(pool))
        .and_then(read_entry)
}


// Handlers
async fn new_entry(entry: Entry, pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    match db::write_entry(&pool, &entry).await {
        Ok(_) => return Ok(http::StatusCode::CREATED),
        Err(_) => return Ok(http::StatusCode::BAD_REQUEST)
    };
}

async fn read_entry(id: i32, pool: SqlitePool) -> Result<warp::reply::Response, Infallible> {
    match db::read_entry(&pool, id).await {
        Ok(entry) => {
            return Ok(warp::reply::json(&entry).into_response())
        },
        Err(_) => return Ok(
            warp::reply::with_status(
                "Invalid id",
                http::StatusCode::BAD_REQUEST,
            ).into_response()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{Fake, Faker};
    use bytes::Bytes;

    #[tokio::test]
    async fn test_get_entry() -> Result<()> {
        let pool = db::tests::setup_test_db().await?;
        db::tests::setup_entries_table(&pool).await?;

        let mut exp_entry: db::Entry = Faker.fake();
        exp_entry.id = Some(1);
        db::write_entry(&pool, &exp_entry).await?;

        let filter = get_entry(pool);

        let res = warp::test::request()
            .method("GET")
            .path("/entry/1")
            .reply(&filter).await;

        let exp_json = Bytes::from(serde_json::to_string(&exp_entry).unwrap());

        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), &exp_json);

        Ok(())
    }

    #[tokio::test]
    async fn test_post_entry() -> Result<()> {
        let pool = db::tests::setup_test_db().await?;
        db::tests::setup_entries_table(&pool).await?;

        let mut exp_entry: db::Entry = Faker.fake();
        exp_entry.id = Some(1);
        
        let exp_json = Bytes::from(serde_json::to_string(&exp_entry).unwrap());

        // db::write_entry(&pool, &exp_entry).await?;

        let filter = post_entry(pool.clone());

        let res = warp::test::request()
            .method("POST")
            .path("/entry")
            .body(&exp_json)
            .reply(&filter).await;

        assert_eq!(res.status(), 201);

        let entry = db::read_entry(&pool, exp_entry.id.unwrap()).await?;

        assert_eq!(&entry, &exp_entry);

        Ok(())
    }
}