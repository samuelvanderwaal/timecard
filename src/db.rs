#![allow(dead_code)]

use dotenv::dotenv;
use std::env;

use anyhow::{Context, Result};
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqliteQueryAs;

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub id: Option<i32>,
    pub start: String,
    pub stop: String,
    pub week_day: String,
    pub code: String,
    pub memo: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub id: Option<i32>,
    pub name: String,
    pub code: String,
}

pub async fn setup_pool() -> Result<SqlitePool> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").context("DATABASE_URL env var must be set!")?;

    Ok(SqlitePool::new(&db_url).await?)
}

pub async fn read_entry(pool: &SqlitePool, id: i32) -> Result<Entry> {
    Ok(
        sqlx::query_as!(Entry, "select * from entries where id = ?", id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn read_all_entries(pool: &SqlitePool) -> Result<Vec<Entry>> {
    Ok(sqlx::query_as!(Entry, "select * from entries")
        .fetch_all(pool)
        .await?)
}

pub async fn write_entry(pool: &SqlitePool, entry: &Entry) -> Result<i32> {
    sqlx::query!(
        "INSERT INTO entries(start, stop, week_day, code, memo) VALUES(?, ?, ?, ?, ?)",
        entry.start,
        entry.stop,
        entry.week_day,
        entry.code,
        entry.memo
    )
    .execute(pool)
    .await?;

    let rec: (i32,) = sqlx::query_as("SELECT last_insert_rowid()")
        .fetch_one(pool)
        .await?;

    Ok(rec.0)
}

pub async fn update_entry(pool: &SqlitePool, entry: &Entry) -> Result<()> {
    sqlx::query!(
        "UPDATE entries SET start=?, stop=?, week_day=?, code=?, memo=?
        WHERE id=?",
        entry.start,
        entry.stop,
        entry.week_day,
        entry.code,
        entry.memo,
        entry.id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_entry(pool: &SqlitePool, id: i32) -> Result<()> {
    sqlx::query!("DELETE FROM entries WHERe id=?", id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn read_project(pool: &SqlitePool, id: i32) -> Result<Project> {
    Ok(
        sqlx::query_as!(Project, "select * from projects where id = ?", id)
            .fetch_one(pool)
            .await?,
    )
}

pub async fn read_all_projects(pool: &SqlitePool) -> Result<Vec<Project>> {
    Ok(sqlx::query_as!(Project, "select * from projects")
        .fetch_all(pool)
        .await?)
}

pub async fn write_project(pool: &SqlitePool, project: &Project) -> Result<i32> {
    sqlx::query!(
        "INSERT INTO projects(name, code) VALUES(?, ?)",
        project.name,
        project.code,
    )
    .execute(pool)
    .await?;

    let rec: (i32,) = sqlx::query_as("SELECT last_insert_rowid()")
        .fetch_one(pool)
        .await?;

    Ok(rec.0)
}

pub async fn update_project(pool: &SqlitePool, project: &Project) -> Result<()> {
    sqlx::query!(
        "UPDATE projects SET name=?, code=?
        WHERE id=?",
        project.name,
        project.code,
        project.id,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_project(pool: &SqlitePool, id: i32) -> Result<()> {
    sqlx::query!("DELETE FROM projects WHERe id=?", id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    async fn setup_db() -> Result<SqlitePool> {
        let db_name: String = random_name();
        let pool = SqlitePool::new(&format!("sqlite:///tmp/{}_test.db", db_name)).await?;

        Ok(pool)
    }

    async fn setup_entries_table(pool: &SqlitePool) -> Result<()> {
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

    async fn setup_projects_table(pool: &SqlitePool) -> Result<()> {
        sqlx::query(
            "CREATE TABLE projects(
                id INTEGER PRIMARY KEY,
                name TEXT,
                code TEXT)",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    fn random_name() -> String {
        thread_rng().sample_iter(&Alphanumeric).take(10).collect()
    }

    #[tokio::test]
    async fn test_write_and_read_entry() -> Result<()> {
        let pool = setup_db().await?;
        setup_entries_table(&pool).await?;

        let mut exp_entry = Entry {
            id: None,
            start: "0900".to_string(),
            stop: "1000".to_string(),
            week_day: "WED".to_string(),
            code: "20-008".to_string(),
            memo: "work, work, work".to_string(),
        };

        let id = write_entry(&pool, &exp_entry).await?;
        exp_entry.id = Some(id);

        let entry = read_entry(&pool, id).await?;
        assert_eq!(entry, exp_entry);

        Ok(())
    }

    #[tokio::test]
    async fn test_read_all_entries() -> Result<()> {
        let pool = setup_db().await?;
        setup_entries_table(&pool).await?;

        let mut exp_entry1 = Entry {
            id: None,
            start: "0900".to_string(),
            stop: "1000".to_string(),
            week_day: "WED".to_string(),
            code: "20-008".to_string(),
            memo: "work, work, work".to_string(),
        };

        let mut exp_entry2 = Entry {
            id: None,
            start: "1200".to_string(),
            stop: "1430".to_string(),
            week_day: "FRI".to_string(),
            code: "20-000".to_string(),
            memo: "work, work, work".to_string(),
        };

        let id1 = write_entry(&pool, &exp_entry1).await?;
        let id2 = write_entry(&pool, &exp_entry2).await?;

        exp_entry1.id = Some(id1);
        exp_entry2.id = Some(id2);

        let entries = read_all_entries(&pool).await?;

        assert_eq!(entries[0], exp_entry1);
        assert_eq!(entries[1], exp_entry2);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_entry() -> Result<()> {
        let pool = setup_db().await?;
        setup_entries_table(&pool).await?;

        let mut exp_entry = Entry {
            id: None,
            start: "0900".to_string(),
            stop: "1000".to_string(),
            week_day: "WED".to_string(),
            code: "20-008".to_string(),
            memo: "work, work, work".to_string(),
        };

        let id = write_entry(&pool, &exp_entry).await?;
        exp_entry.id = Some(id);

        let entry = read_entry(&pool, id).await?;
        assert_eq!(entry.week_day, exp_entry.week_day);

        exp_entry.week_day = "THU".to_string();
        update_entry(&pool, &exp_entry).await?;

        let entry = read_entry(&pool, id).await?;
        assert_eq!(entry.week_day, exp_entry.week_day);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_entry() -> Result<()> {
        let pool = setup_db().await?;
        setup_entries_table(&pool).await?;

        let mut exp_entry = Entry {
            id: None,
            start: "0900".to_string(),
            stop: "1000".to_string(),
            week_day: "WED".to_string(),
            code: "20-008".to_string(),
            memo: "work, work, work".to_string(),
        };

        let id = write_entry(&pool, &exp_entry).await?;
        exp_entry.id = Some(id);

        delete_entry(&pool, id).await?;
        assert!(read_entry(&pool, id).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_write_and_read_project() -> Result<()> {
        let pool = setup_db().await?;
        setup_projects_table(&pool).await?;

        let mut exp_project = Project {
            id: None,
            name: "PPP".to_string(),
            code: "20-008".to_string(),
        };

        let id = write_project(&pool, &exp_project).await?;
        exp_project.id = Some(id);

        let project = read_project(&pool, id).await?;
        assert_eq!(project, exp_project);

        Ok(())
    }

    #[tokio::test]
    async fn test_read_all_projects() -> Result<()> {
        let pool = setup_db().await?;
        setup_projects_table(&pool).await?;

        let mut exp_project1 = Project {
            id: None,
            name: "PPP".to_string(),
            code: "20-008".to_string(),
        };

        let mut exp_project2 = Project {
            id: None,
            name: "General".to_string(),
            code: "20-000-00".to_string(),
        };

        let id1 = write_project(&pool, &exp_project1).await?;
        let id2 = write_project(&pool, &exp_project2).await?;

        exp_project1.id = Some(id1);
        exp_project2.id = Some(id2);

        let projects = read_all_projects(&pool).await?;

        assert_eq!(projects[0], exp_project1);
        assert_eq!(projects[1], exp_project2);

        Ok(())
    }

    #[tokio::test]
    async fn test_update_project() -> Result<()> {
        let pool = setup_db().await?;
        setup_projects_table(&pool).await?;

        let mut exp_project = Project {
            id: None,
            name: "PPP".to_string(),
            code: "20-008".to_string(),
        };

        let id = write_project(&pool, &exp_project).await?;
        exp_project.id = Some(id);

        let project = read_project(&pool, id).await?;
        assert_eq!(project.name, project.name);

        exp_project.name = "New name".to_string();
        update_project(&pool, &exp_project).await?;

        let project = read_project(&pool, id).await?;
        assert_eq!(project.name, exp_project.name);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_project() -> Result<()> {
        let pool = setup_db().await?;
        setup_projects_table(&pool).await?;

        let mut exp_project = Project {
            id: None,
            name: "PPP".to_string(),
            code: "20-008".to_string(),
        };

        let id = write_project(&pool, &exp_project).await?;
        exp_project.id = Some(id);

        delete_project(&pool, id).await?;
        assert!(read_project(&pool, id).await.is_err());

        Ok(())
    }

}
