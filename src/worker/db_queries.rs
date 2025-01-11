use std::time::Duration;

use crate::scraper::{Product, Scraper};
use sqlx::Row;
use sqlx::SqlitePool;

pub struct ProductWithoutInfoDb {
    pub id: i64,
    pub name: String,
}

pub async fn get_products_without_beef_cut(
    pool: &SqlitePool,
) -> Result<Vec<ProductWithoutInfoDb>, sqlx::Error> {
    let res = sqlx::query(
        r#"
        SELECT id, name FROM product 
        WHERE beef_cut is null
        LIMIT 10
    "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(res
        .iter()
        .map(|r| ProductWithoutInfoDb {
            id: r.get(0),
            name: r.get(1),
        })
        .collect())
}

pub async fn insert_beef_cut(
    pool: &SqlitePool,
    product_id: i64,
    beef_cut: &str,
    confidence: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        update product set beef_cut=?, beef_cut_guess_confidence=?
        where id=?
    "#,
    )
    .bind(beef_cut)
    .bind(confidence)
    .bind(product_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_products_without_embedings(
    pool: &SqlitePool,
) -> Result<Vec<ProductWithoutInfoDb>, sqlx::Error> {
    let res = sqlx::query(
        r#"
        SELECT id, name FROM product 
        WHERE embedding is null
        LIMIT 10
    "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(res
        .iter()
        .map(|r| ProductWithoutInfoDb {
            id: r.get(0),
            name: r.get(1),
        })
        .collect())
}

pub async fn insert_embedding(
    pool: &SqlitePool,
    product_id: i64,
    embedding: &[u8],
    embedding_model: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        update product set embedding=?, embedding_model=?
        where id=?
    "#,
    )
    .bind(embedding)
    .bind(embedding_model)
    .bind(product_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn insert_or_get_product(
    trx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    product: &Product,
) -> Result<i64, sqlx::Error> {
    let product_id: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT id FROM product 
        WHERE name = ?
    "#,
    )
    .bind(&product.name)
    .fetch_optional(&mut **trx)
    .await?;

    if let Some(id) = product_id {
        return Ok(id);
    }

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
    Ok(product_id)
}

pub async fn insert_product(
    trx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    scraper: &Scraper,
    product: &Product,
) -> Result<(i64, i64), sqlx::Error> {
    let product_id: i64 = insert_or_get_product(trx, product).await?;

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
    trx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    scraper: &Scraper,
    products: &[Product],
) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    let mut inserted = vec![];

    for product in products.iter() {
        let ids = insert_product(trx, scraper, product).await?;
        inserted.push(ids);
    }
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
    trx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
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
    .fetch_one(&mut **trx)
    .await
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use sqlx::Acquire;

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
        let mut trx = pool.begin().await;
        let _id = insert_run(&mut trx, started, finished).await.unwrap();
        let _id2 = insert_run(&mut trx, started2, finished2).await.unwrap();
        trx.commit().await.unwrap();
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
        let mut trx = pool.begin().await; 
        let inserted = insert_products(&mut trx, &scrapers[0], &product)
            .await
            .unwrap();

        trx.commit().await.unwrap();

        assert_eq!(vec![(1, 1), (2, 2)], inserted)
    }
}
