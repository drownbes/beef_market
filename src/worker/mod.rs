mod db_queries;
use crate::clock::Clock;
use crate::ollama::OllamaRunner;
use crate::scraper::Scraper;
use db_queries::{
    get_latest_run, get_products_without_beef_cut, get_products_without_embedings, insert_beef_cut,
    insert_embedding, insert_products, insert_run,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::info;
use zerocopy::IntoBytes;

pub struct Worker {
    clock: Arc<Mutex<dyn Clock>>,
    pool: SqlitePool,
    ollama: Arc<Mutex<OllamaRunner>>,
    scrapers: Vec<Scraper>,
}

impl Worker {
    pub fn new(
        clock: Arc<Mutex<dyn Clock>>,
        pool: SqlitePool,
        ollama: Arc<Mutex<OllamaRunner>>,
        scrapers: Vec<Scraper>,
    ) -> Worker {
        Worker {
            clock,
            pool,
            ollama,
            scrapers,
        }
    }

    async fn is_time_to_run(&self) -> Result<bool, sqlx::Error> {
        let last_run_ts = get_latest_run(&self.pool).await?;
        let now: Duration = self.clock.lock().await.utc();

        match last_run_ts {
            Some(last_ts) => Ok(now.checked_sub(last_ts).expect("Time goes back")
                >= Duration::from_secs(24 * 60 * 60)),
            None => Ok(true),
        }
    }

    pub async fn worker_loop(&self) -> anyhow::Result<()> {
        info!("Starting worker loop");
        loop {
            if self.is_time_to_run().await? {
                info!("Time to do scraping");
                self.do_scraping().await?;
            }

            self.do_embeddins().await?;
            self.do_beef_cut_detection().await?;
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }

    async fn do_scraping(&self) -> anyhow::Result<()> {
        let started: Duration = self.clock.lock().await.utc();
        for scraper in self.scrapers.iter() {
            let products = scraper.run().await?;
            insert_products(&self.pool, scraper, &products).await?;
        }
        let finished: Duration = self.clock.lock().await.utc();
        insert_run(&self.pool, started, finished).await?;
        info!("Scraping finished");
        Ok(())
    }

    async fn do_embeddins(&self) -> anyhow::Result<()> {
        info!("Embedding processing started");
        loop {
            let prds = get_products_without_embedings(&self.pool).await?;

            info!("Found products {} without empbedding", prds.len());
            if prds.is_empty() {
                break;
            }

            for product in prds {
                let ollama = self.ollama.lock().await;
                let embedding = ollama.create_embedding(&product.name).await?;

                info!("Processed embedding for {}", &product.name);
                insert_embedding(
                    &self.pool,
                    product.id,
                    embedding.as_bytes(),
                    &ollama.embedding_model,
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn do_beef_cut_detection(&self) -> anyhow::Result<()> {
        info!("Beef cut detection started");
        loop {
            let prds = get_products_without_beef_cut(&self.pool).await?;

            info!("Found products {} without beef_cut", prds.len());
            if prds.is_empty() {
                break;
            }

            for product in prds {
                let ollama = self.ollama.lock().await;
                if let Some((cut, confidence)) = ollama.guess_beef_cut(&product.name).await? {
                    info!("Processed beef_cut for {}", &product.name);
                    insert_beef_cut(&self.pool, product.id, &cut, confidence).await?;
                }
            }
        }

        Ok(())
    }
}
