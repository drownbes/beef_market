use libsqlite3_sys::sqlite3_auto_extension;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;
use sqlite_vec::sqlite3_vec_init;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::{ConnectOptions, Pool, Sqlite};
use sqlx::Connection;
use sqlx::Row;
use std::str::FromStr;
use zerocopy::IntoBytes;
use sqlx::Executor;

mod product;
mod beef_cut;
mod product_history;

pub async fn get_sqlite_pool() -> sqlx::Result<Pool<Sqlite>>{
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    SqlitePoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| Box::pin(async move {
            conn.execute("PRAGMA journal_mode=WAL;")
            .await?;
            Ok(())
        }))
        .connect("sqlite://db.db")
        
        .await
}

async fn run() -> anyhow::Result<()> {
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    let mut conn = SqliteConnectOptions::from_str("sqlite::memory:")?
        .journal_mode(SqliteJournalMode::Wal)
        .connect()
        .await?;

    sqlx::query(
        "create table embeds(
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        text TEXT,
        embedding FLOAT[1024]
    )",
    )
    .execute(&mut conn)
    .await?;

    let ollama = Ollama::default();

    let apple = "striploin steak";
    let orange = "entrecote steak";
    let car = "beef burger";

    let words = vec![apple, orange, car, "horse dick stew"];
    let mut ns = vec![];

    for word in words {
        let request =
            GenerateEmbeddingsRequest::new("snowflake-arctic-embed2".to_string(), word.into());
        let res = ollama.generate_embeddings(request).await.unwrap();
        ns.push((word, res.embeddings[0].clone()));
    }

    let mut tx = conn.begin().await?;
    //
    //
    for (t, e) in ns.iter() {
        let v: &[u8] = e.as_bytes();
        sqlx::query("insert into embeds(text, embedding) values(?, ?)")
            .bind(t)
            .bind(v)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    let v: &[u8] = ns[0].1.as_bytes();

    let rr = sqlx::query(
        "
        select text, 
            vec_distance_L2(embedding, ?) as distance,
            vec_distance_L1(embedding, ?),
            vec_distance_cosine(embedding, ?)
        from embeds",
    )
    .bind(v)
    .bind(v)
    .bind(v)
    .fetch_all(&mut conn)
    .await?;

    for r in rr {
        let ver: String = r.get(0);
        let l2: f32 = r.get(1);
        let l1: f32 = r.get(1);
        let cosine: f32 = r.get(1);
        println!("{} {} {} {}", ver, l2, l1, cosine);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_db() {
        let r = run().await;
        dbg!(r);
    }
}
