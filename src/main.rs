use std::{env, path::Path, sync::Arc};

use beef_market::{
    clock::DefaultClock,
    config::read_config,
    db::{get_sqlite_pool, run_migrations},
    logger::init_tracing,
    ollama::OllamaRunner,
    scraper::get_scrapers,
    worker::Worker,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    let cfg_path = Path::new(&args[1]);
    let config = read_config(cfg_path)?;
    dbg!(&config);
    init_tracing();
    let pool = get_sqlite_pool(&config.db.conn_str).await?;
    run_migrations(&pool).await?;
    let ollama_runner = Arc::new(Mutex::new(OllamaRunner::new(&config.ollama)));
    let scrapers = get_scrapers(&pool).await?;
    let clock = Arc::new(Mutex::new(DefaultClock));

    let worker = Worker::new(clock, pool, ollama_runner, scrapers);

    let res = tokio::spawn(async move { worker.worker_loop().await }).await;

    dbg!(&res);

    Ok(())
}
