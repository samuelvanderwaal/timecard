// Std
use std::convert::Infallible;

// Crates
use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use tracing::{info};
use warp::reply::Reply;
use warp::{http, Filter};

// Modules
use crate::db;
use crate::{Entry, Project};

fn json_body_entry() -> impl Filter<Extract = (Entry,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn json_body_project() -> impl Filter<Extract = (Project,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn with_pool(
    pool: SqlitePool,
) -> impl Filter<Extract = (SqlitePool,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || pool.clone())
}

// Filters
pub fn post_entry(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("entry")
        .and(warp::post())
        .and(json_body_entry())
        .and(with_pool(pool))
        .and_then(new_entry)
}

pub fn get_entry(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        // .and(warp::path!("entry" / i32))
        .and(warp::path("entry"))
        .and(warp::path::param::<i32>())
        .and(with_pool(pool))
        .and_then(read_entry)
}

pub fn get_entries_between(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path!("entries_between" / String / String))
        .and(with_pool(pool))
        .and_then(entries_between)
}

pub fn read_last_entry(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("last_entry"))
        .and(with_pool(pool))
        .and_then(last_entry)
}

pub fn update_entry(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("update_entry"))
        .and(json_body_entry())
        .and(with_pool(pool))
        .and_then(update_entry_handler)
}

pub fn delete_entry(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("delete_entry"))
        .and(warp::path::param::<i32>())
        .and(with_pool(pool))
        .and_then(delete_entry_handler)
}

pub fn delete_last_entry(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("delete_last_entry"))
        .and(with_pool(pool))
        .and_then(delete_last_entry_handler)
}

pub fn post_project(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("project")
        .and(warp::post())
        .and(json_body_project())
        .and(with_pool(pool))
        .and_then(new_project)
}

pub fn get_project(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("project"))
        .and(warp::path::param::<i32>())
        .and(with_pool(pool))
        .and_then(read_project)
}

pub fn get_all_projects(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("all_projects"))
        .and(with_pool(pool))
        .and_then(read_all_projects)
}

pub fn update_project(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("update_project"))
        .and(json_body_project())
        .and(with_pool(pool))
        .and_then(update_project_handler)
}

pub fn delete_project(
    pool: SqlitePool,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path("delete_project"))
        .and(warp::path::param::<String>())
        .and(with_pool(pool))
        .and_then(delete_project_handler)
}

// Handlers
async fn new_entry(entry: Entry, pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    info!("Processing new entry");
    match db::write_entry(&pool, &entry).await {
        Ok(_) => return Ok(http::StatusCode::OK),
        Err(_) => return Ok(http::StatusCode::BAD_REQUEST),
    };
}

async fn read_entry(id: i32, pool: SqlitePool) -> Result<warp::reply::Response, Infallible> {
    info!("Reading entry #{}", id);
    match db::read_entry(&pool, id).await {
        Ok(entry) => return Ok(warp::reply::json(&entry).into_response()),
        Err(_) => {
            return Ok(
                warp::reply::with_status("Invalid id", http::StatusCode::BAD_REQUEST)
                    .into_response(),
            )
        }
    }
}

async fn entries_between(
    start: String,
    stop: String,
    pool: SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    info!("Reading entries between {} and {}", start, stop);
    match db::read_entries_between(&pool, start, stop).await {
        Ok(entries) => return Ok(warp::reply::json(&entries).into_response()),
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Invalid date range",
                http::StatusCode::BAD_REQUEST,
            )
            .into_response())
        }
    }
}

async fn last_entry(pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    info!("Reading most recent entry.");
    match db::read_last_entry(&pool).await {
        Ok(entry) => return Ok(warp::reply::json(&entry).into_response()),
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Failed to read last entry.",
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response())
        }
    }
}

async fn update_entry_handler(
    entry: Entry,
    pool: SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    info!("Reading most recent entry.");
    match db::update_entry(&pool, &entry).await {
        Ok(_) => return Ok(http::StatusCode::OK),
        Err(_) => return Ok(http::StatusCode::BAD_REQUEST),
    }
}

async fn delete_entry_handler(id: i32, pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    info!("Deleting entry #{}", id);
    match db::delete_entry(&pool, id).await {
        Ok(_) => return Ok(http::StatusCode::OK),
        Err(_) => return Ok(http::StatusCode::BAD_REQUEST),
    }
}

async fn delete_last_entry_handler(pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    info!("Deleting most recent entry.");
    match db::delete_last_entry(&pool).await {
        Ok(_) => Ok(http::StatusCode::OK),
        Err(_) => Ok(http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn new_project(project: Project, pool: SqlitePool) -> Result<impl warp::Reply, Infallible> {
    info!("Creating a new project.");
    match db::write_project(&pool, &project).await {
        Ok(_) => return Ok(http::StatusCode::OK),
        Err(_) => return Ok(http::StatusCode::BAD_REQUEST),
    };
}

async fn read_project(id: i32, pool: SqlitePool) -> Result<warp::reply::Response, Infallible> {
     info!("Reading project #{}", id);
    match db::read_project(&pool, id).await {
        Ok(project) => return Ok(warp::reply::json(&project).into_response()),
        Err(_) => {
            return Ok(
                warp::reply::with_status("Invalid id", http::StatusCode::BAD_REQUEST)
                    .into_response(),
            )
        }
    }
}

async fn read_all_projects(pool: SqlitePool) -> Result<warp::reply::Response, Infallible> {
    info!("Reading all projects.");
    match db::read_all_projects(&pool).await {
        Ok(projects) => return Ok(warp::reply::json(&projects).into_response()),
        Err(_) => {
            return Ok(
                warp::reply::with_status("Invalid id", http::StatusCode::BAD_REQUEST)
                    .into_response(),
            )
        }
    }
}

async fn update_project_handler(
    project: Project,
    pool: SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    info!("Updating project.");
    match db::update_project(&pool, &project).await {
        Ok(_) => return Ok(http::StatusCode::OK),
        Err(_) => return Ok(http::StatusCode::BAD_REQUEST),
    }
}

async fn delete_project_handler(
    code: String,
    pool: SqlitePool,
) -> Result<impl warp::Reply, Infallible> {
    info!("Deleting project: {}", code);
    match db::delete_project(&pool, code).await {
        Ok(_) => {
            return Ok(warp::reply::with_status(
                "Entry deleted.",
                http::StatusCode::OK,
            ))
        }
        Err(_) => {
            return Ok(warp::reply::with_status(
                "Error deleting entry.",
                http::StatusCode::BAD_REQUEST,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use fake::{Fake, Faker};
    use serde_json;

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
            .reply(&filter)
            .await;

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
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);

        let entry = db::read_entry(&pool, exp_entry.id.unwrap()).await?;

        assert_eq!(&entry, &exp_entry);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_entry() -> Result<()> {
        let pool = db::tests::setup_test_db().await?;
        db::tests::setup_entries_table(&pool).await?;

        let mut exp_entry: db::Entry = Faker.fake();
        let id = db::write_entry(&pool, &exp_entry).await?;

        exp_entry.id = Some(id);
        exp_entry.start = String::from("0900");
        exp_entry.stop = String::from("1100");
        exp_entry.code = String::from("20-008");
        exp_entry.memo = String::from("work, work, work");

        let exp_json = Bytes::from(serde_json::to_string(&exp_entry).unwrap());

        let filter = update_entry(pool.clone());

        let res = warp::test::request()
            .method("POST")
            .path("/update_entry")
            .body(&exp_json)
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);

        let entry = db::read_entry(&pool, exp_entry.id.unwrap()).await?;

        assert_eq!(&entry, &exp_entry);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_entry() -> Result<()> {
        let pool = db::tests::setup_test_db().await?;
        db::tests::setup_entries_table(&pool).await?;

        let mut entry: db::Entry = Faker.fake();
        entry.id = Some(1);
        db::write_entry(&pool, &entry).await?;

        let filter = delete_entry(pool.clone());

        let res = warp::test::request()
            .method("POST")
            .path("/delete_entry/1")
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);

        // Entry should not exist.
        let _ = db::read_entry(&pool, entry.id.unwrap()).await.is_err();

        assert_eq!(res.body(), "Entry deleted.");

        Ok(())
    }

    #[tokio::test]
    async fn test_get_project() -> Result<()> {
        let pool = db::tests::setup_test_db().await?;
        db::tests::setup_projects_table(&pool).await?;

        let mut exp_project: db::Project = Faker.fake();
        exp_project.id = Some(1);
        db::write_project(&pool, &exp_project).await?;

        let filter = get_project(pool);

        let res = warp::test::request()
            .method("GET")
            .path("/project/1")
            .reply(&filter)
            .await;

        let exp_json = Bytes::from(serde_json::to_string(&exp_project).unwrap());

        assert_eq!(res.status(), 200);
        assert_eq!(res.body(), &exp_json);

        Ok(())
    }

    #[tokio::test]
    async fn test_post_project() -> Result<()> {
        let pool = db::tests::setup_test_db().await?;
        db::tests::setup_projects_table(&pool).await?;

        let mut exp_project: db::Project = Faker.fake();
        exp_project.id = Some(1);

        let exp_json = Bytes::from(serde_json::to_string(&exp_project).unwrap());

        let filter = post_project(pool.clone());

        let res = warp::test::request()
            .method("POST")
            .path("/project")
            .body(&exp_json)
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);

        let project = db::read_project(&pool, exp_project.id.unwrap()).await?;

        assert_eq!(&project, &exp_project);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_project() -> Result<()> {
        let pool = db::tests::setup_test_db().await?;
        db::tests::setup_projects_table(&pool).await?;

        let mut exp_project: db::Project = Faker.fake();
        let id = db::write_project(&pool, &exp_project).await?;

        exp_project.id = Some(id);
        exp_project.name = String::from("General Support");
        exp_project.code = String::from("20-008");

        let exp_json = Bytes::from(serde_json::to_string(&exp_project).unwrap());

        let filter = update_project(pool.clone());

        let res = warp::test::request()
            .method("POST")
            .path("/update_project")
            .body(&exp_json)
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);

        let project = db::read_project(&pool, exp_project.id.unwrap()).await?;

        assert_eq!(&project, &exp_project);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project() -> Result<()> {
        let pool = db::tests::setup_test_db().await?;
        db::tests::setup_projects_table(&pool).await?;

        let mut project: db::Project = Faker.fake();
        project.id = Some(1);
        let code = project.code.clone();
        db::write_project(&pool, &project).await?;

        let filter = delete_project(pool.clone());

        let res = warp::test::request()
            .method("POST")
            .path(&format!("/delete_project/{}", &code))
            .reply(&filter)
            .await;

        assert_eq!(res.status(), 200);

        // Entry should not exist.
        let _ = db::read_project(&pool, project.id.unwrap()).await.is_err();

        assert_eq!(res.body(), "Entry deleted.");

        Ok(())
    }
}
