use std::path::Path;

use sqlx::migrate::Migrator;

use beef_market::db::get_sqlite_pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let m = Migrator::new(Path::new("./migrations")).await?;
    let pool = get_sqlite_pool("sqlite://db.db").await?;

    m.run(&pool).await?;
    Ok(())
}
