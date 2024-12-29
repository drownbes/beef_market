use futures::future::join_all;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::time::Duration;

use sqlx::Row;
use sqlx::{Pool, Sqlite, SqlitePool};

use crate::clock::Clock;
use crate::config::AppConfig;
use crate::scraper::{self, Scraper};


struct Worker {
    clock: Arc<Mutex<dyn Clock>>,
    pool: SqlitePool,
    scrapers: Vec<Scraper>
}

impl Worker {
    async fn is_time_to_run(&self) -> Result<bool, sqlx::Error> {
        let last_run_ts  = get_latest_run(&self.pool).await?;
        let now : Duration = self.clock.lock().await.utc();

        match last_run_ts {
            Some(last_ts) => Ok(now.checked_sub(last_ts).expect("Time goes back") >= Duration::from_secs(24*60*60)),
            None => Ok(true)
        }
    }

    async fn worker_loop(&self) -> anyhow::Result<()> {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            if self.is_time_to_run().await? {
                self.do_work().await?;
            }
        }
    }

    async fn do_work(&self) -> anyhow::Result<()> {
        for scraper in self.scrapers.iter() {
            let prs = scraper.run().await?;
        }
        Ok(())
    }
}



async fn get_latest_run(pool: &SqlitePool) -> Result<Option<Duration>, sqlx::Error> {
    let r = sqlx::query(
        r#"
            select finished_at 
            from worker_run 
            order by finished_at desc
            limit 1
            "#,
    )
    .fetch_optional(pool)
    .await?;
    Ok(r.map(|row| Duration::from_secs(row.get(0))))
}

async fn insert_run(pool: &SqlitePool, started: Duration, finished: Duration) -> Result<u64, sqlx::Error> {
    let started : i64 = started.as_secs().try_into().unwrap();
    let finished: i64 = finished.as_secs().try_into().unwrap();
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
    use crate::{clock::DefaultClock, db::{get_sqlite_pool, run_migrations}};

    use std::time::Duration;

    use super::*;

    async fn create_pool() -> anyhow::Result<SqlitePool> {
        let pool = get_sqlite_pool("sqlite::memory:").await.unwrap();
        run_migrations(&pool).await.unwrap();
        Ok(pool)
    }

    #[tokio::test]
    async fn test_worker_db() {
        let pool = create_pool().await.unwrap();
        let c = DefaultClock {};
        let started = c.utc();
        let finished = started.saturating_add(Duration::from_secs(10));
        let started2 = started.saturating_add(Duration::from_secs(4000));
        let finished2 = started2.saturating_add(Duration::from_secs(10));
        let _id = insert_run(&pool, started, finished).await.unwrap();
        let _id2 = insert_run(&pool, started2, finished2).await.unwrap();
        let latest_finish = get_latest_run(&pool).await.unwrap();

        assert_eq!(latest_finish.unwrap().as_secs(), finished2.as_secs());
    }
}
