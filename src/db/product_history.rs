use std::time::Duration;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, FromRow)]
struct ProductHistory {
    id: Option<u64>,
    shop_id: i64,
    product_id: i64,
    price: i64,
    inserted_at: Duration,
}

impl ProductHistory {
    async fn insert(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let inserted_id: u64 = sqlx::query_scalar(
            r#"
            INSERT INTO product_history (shop_id, product_id, price)
            VALUES ($1, $2, $3)
            RETURNING id
            "#,
        )
        .bind(self.shop_id)
        .bind(self.product_id)
        .bind(self.price)
        .fetch_one(pool)
        .await?;

        Ok(inserted_id)
    }
}
