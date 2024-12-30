use std::time::Duration;

use crate::scraper::{Product, Scraper};
use sqlx::Row;
use sqlx::SqlitePool;

pub async fn insert_product(
    trx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    scraper: &Scraper,
    product: &Product,
) -> Result<(i64, i64), sqlx::Error> {
    let product_id: i64 = sqlx::query_scalar(
        r#"
            INSERT INTO product (name)
            VALUES (?)
            RETURNING id
            "#,
    )
    .bind(&product.name)
    .fetch_one(&mut **trx)
    .await?;

    let product_history_id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO product_history (shop_id, product_id, price)
        VALUES (?, ?, ?) RETURNING id
        "#,
    )
    .bind(scraper.id)
    .bind(product_id)
    .bind(&product.price)
    .fetch_one(&mut **trx)
    .await?;
    Ok((product_id, product_history_id))
}

pub async fn insert_products(
    pool: &SqlitePool,
    scraper: &Scraper,
    products: &[Product],
) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    let mut trx = pool.begin().await?;

    let mut inserted = vec![];

    for product in products.iter() {
        let ids = insert_product(&mut trx, scraper, product).await?;
        inserted.push(ids);
    }

    trx.commit().await?;

    Ok(inserted)
}

pub async fn get_latest_run(pool: &SqlitePool) -> Result<Option<Duration>, sqlx::Error> {
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

pub async fn insert_run(
    pool: &SqlitePool,
    started: Duration,
    finished: Duration,
) -> Result<u64, sqlx::Error> {
    let started: i64 = started.as_secs().try_into().unwrap();
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
    use rust_decimal_macros::dec;

    use crate::{
        clock::{Clock, DefaultClock},
        db::{get_sqlite_pool, run_migrations},
        scraper::{get_scrapers, PriceEur},
    };

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

    #[tokio::test]
    async fn test_insert_product() {
        let pool = create_pool().await.unwrap();
        let mut trx = pool.begin().await.unwrap();

        let scrapers = get_scrapers(&pool).await.unwrap();

        let product = Product {
            name: "test product".into(),
            price: PriceEur(dec!(10.24)),
        };
        let ids = insert_product(&mut trx, &scrapers[0], &product)
            .await
            .unwrap();
        trx.commit().await.unwrap();

        assert_eq!((1, 1), ids);
    }

    #[tokio::test]
    async fn test_insert_products() {
        let pool = create_pool().await.unwrap();

        let scrapers = get_scrapers(&pool).await.unwrap();

        let product = vec![
            Product {
                name: "test product".into(),
                price: PriceEur(dec!(10.24)),
            },
            Product {
                name: "test product1".into(),
                price: PriceEur(dec!(19.24)),
            },
        ];
        let inserted = insert_products(&pool, &scrapers[0], &product)
            .await
            .unwrap();

        assert_eq!(vec![(1, 1), (2, 2)], inserted)
    }
}
