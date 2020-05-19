#![allow(dead_code)]

use dotenv::dotenv;
use std::env;

use anyhow::{Context, Result};
// use sqlx::prelude::*;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqliteQueryAs;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: i32,
    pub start: String,
    pub stop: String,
    pub week_day: String,
    pub code: String,
    pub memo: String,
}

pub struct NewEntry {
    pub start: String,
    pub stop: String,
    pub week_day: String,
    pub code: String,
    pub memo: String,
}

impl NewEntry {
    pub fn into_entry(self, id: i32) -> Entry {
        Entry{
            id,
            start: self.start,
            stop: self.stop,
            week_day: self.week_day,
            code: self.code,
            memo: self.memo,
        }
    }
}

pub async fn setup_conn() -> Result<SqlitePool> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").context("DATABASE_URL env var must be set!")?;

    Ok(SqlitePool::new(&db_url).await?)
}

pub async fn read_entry(pool: &SqlitePool, id: i32) -> Result<Entry> {
    Ok(
        sqlx::query_as!(Entry, "select * from entries where id = ?", id)
            .fetch_one(pool)
            .await?
    )
}

pub async fn read_all_entries(pool: &SqlitePool) -> Result<Vec<Entry>> {
    Ok(sqlx::query_as!(Entry, "select * from entries")
        .fetch_all(pool)
        .await?)
}

pub async fn write_entry(pool: &SqlitePool, entry: &NewEntry) -> Result<i32> {
    sqlx::query!(
        "INSERT INTO entries(start, stop, week_day, code, memo) VALUES(?, ?, ?, ?, ?)",
        entry.start,
        entry.stop,
        entry.week_day,
        entry.code,
        entry.memo
    )
    .execute(pool).await?;

    let rec: (i32,) = sqlx::query_as("SELECT last_insert_rowid()")
        .fetch_one(pool)
        .await?;

    Ok(rec.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    async fn setup_db() -> Result<SqlitePool> {
        let db_name: String = random_name();
        let conn = SqlitePool::new(&format!("sqlite:///tmp/{}_test.db", db_name)).await?;

        Ok(conn)
    }

    async fn setup_entries(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            "CREATE TABLE entries(
                id INTEGER PRIMARY KEY,
                start TEXT,
                stop TEXT,
                week_day TEXT,
                code TEXT,
                memo TEXT)",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    fn random_name() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(10).collect()
    }

    #[async_std::test]
    async fn test_write_and_read_entry() -> Result<()> {
        let pool = setup_db().await?;
        setup_entries(&pool).await?;

        let new_entry = NewEntry{
            start: "0900".to_string(),
            stop: "1000".to_string(),
            week_day: "WED".to_string(),
            code: "20-008".to_string(),
            memo: "work, work, work".to_string(),
        };

        let id = write_entry(&pool, &new_entry).await?;

        let exp_entry: Entry = new_entry.into_entry(id);

        let entry = read_entry(&pool, id).await?;

        assert_eq!(entry, exp_entry);

        Ok(())
    }
}
