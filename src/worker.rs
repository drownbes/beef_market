use chrono::Utc;
use std::time::Duration;

use sqlx::Row;
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::config::AppConfig;

async fn worker_loop(pool: &SqlitePool, config: AppConfig) -> anyhow::Result<()> {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

async fn get_latest_run(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let r = sqlx::query(
        r#"
            select finished_at 
            from worker_run 
            order by finished_at desc
            limit 1
            "#,
    )
    .fetch_one(pool)
    .await?;

    let ts: i64 = r.get(0);
    Ok(ts)
}

async fn insert_run(pool: &SqlitePool, started: i64, finished: i64) -> Result<u64, sqlx::Error> {
    sqlx::query_scalar(
        r#"
        insert into worker_run (started_at, finished_at) values(?, ?)
        returning id
    "#,
    )
    .bind(started)
    .bind(finished)
    .fetch_one(pool)
    .await
}

#[cfg(test)]
mod tests {
    use crate::db::{get_sqlite_pool, run_migrations};

    use super::*;

    async fn create_pool() -> anyhow::Result<SqlitePool> {
        let pool = get_sqlite_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        Ok(pool)
    }

    #[tokio::test]
    async fn test_worker_db() {
        let pool = create_pool().await.unwrap();
        let started = Utc::now().timestamp();
        let finished = started + 10;
        let started2 = started + 4000;
        let finished2 = finished + 4000;
        let _id = insert_run(&pool, started, finished).await.unwrap();
        let _id2 = insert_run(&pool, started2, finished2).await.unwrap();
        let latest_finish = get_latest_run(&pool).await.unwrap();

        assert_eq!(latest_finish, finished2);
    }
}
