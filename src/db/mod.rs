use libsqlite3_sys::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use sqlx::{Pool, Sqlite};
use std::path::Path;
use std::str::FromStr;

pub async fn get_sqlite_pool(conn_str: &str) -> sqlx::Result<Pool<Sqlite>> {
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute::<
            *const (),
            unsafe extern "C" fn(
                *mut libsqlite3_sys::sqlite3,
                *mut *mut i8,
                *const libsqlite3_sys::sqlite3_api_routines,
            ) -> i32,
        >(sqlite3_vec_init as *const ())));
    }

    let options = SqliteConnectOptions::from_str(conn_str)?
        .journal_mode(SqliteJournalMode::Wal)
        .create_if_missing(true);

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
}

pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    let m = Migrator::new(Path::new("./migrations")).await?;
    m.run(pool).await?;
    Ok(())
}
