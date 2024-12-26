use sqlx::{FromRow, SqlitePool};

#[derive(FromRow)]
struct ProductDB {
    id: Option<i64>,
    name: String,
    embedding: Vec<u8>,
    embedding_model: String,
    beef_cut_id: i64,
    beef_cut_guess_confidence: i64,
    inserted_at: u64,
}

impl ProductDB {
    async fn insert(&self, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
        let inserted_id: u64 = sqlx::query_scalar(
            r#"
            INSERT INTO product (name, embedding, embedding_model, beef_cut_id, beef_cut_guess_confidence)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id
            "#,
        )
            .bind(&self.name)
            .bind(&self.embedding)
            .bind(&self.embedding_model)
            .bind(self.beef_cut_id)
            .bind(self.beef_cut_guess_confidence)
            .fetch_one(pool)
        .await?;

        Ok(inserted_id)
    }

    async fn get(pool: &SqlitePool, id: i64) -> Result<Self, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT id, name, embedding, embedding_model, beef_cut_id, beef_cut_guess_confidence, inserted_at
            FROM product
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }
}
