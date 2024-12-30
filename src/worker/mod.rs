mod db_queries;
use crate::clock::Clock;
use crate::scraper::Scraper;
use db_queries::{get_latest_run, insert_products};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

struct Worker {
    clock: Arc<Mutex<dyn Clock>>,
    pool: SqlitePool,
    scrapers: Vec<Scraper>,
}

impl Worker {
    async fn is_time_to_run(&self) -> Result<bool, sqlx::Error> {
        let last_run_ts = get_latest_run(&self.pool).await?;
        let now: Duration = self.clock.lock().await.utc();

        match last_run_ts {
            Some(last_ts) => Ok(now.checked_sub(last_ts).expect("Time goes back")
                >= Duration::from_secs(24 * 60 * 60)),
            None => Ok(true),
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
            let products = scraper.run().await?;
            insert_products(&self.pool, scraper, &products).await?;
        }
        Ok(())
    }
}
