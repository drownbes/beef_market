use sqlx::{FromRow, SqlitePool};

#[derive(FromRow)]
struct BeefCutDB {
    id: Option<i64>,
    name: String
}

impl BeefCutDB {
    async fn insert(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let inserted_id: u64 = sqlx::query_scalar(
            r#"
            INSERT INTO beef_cut (name)
            VALUES ($1)
            RETURNING id
            "#,
        )
            .bind(&self.name)
            .fetch_one(pool)
        .await?;

        Ok(inserted_id)
    }

    async fn get(pool: &SqlitePool, id: i64) -> Result<Self, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, name
            FROM product
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }
}
