use once_cell::sync::Lazy;
use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};
use std::sync::Arc;

use crate::key::Key;

const DB_URL: &str = "sqlite:instant-karma.db";

pub static DB_POOL: Lazy<async_std::sync::Mutex<Option<Arc<Pool<Sqlite>>>>> =
    Lazy::new(|| async_std::sync::Mutex::new(None));

pub async fn initialize() -> Result<(), sqlx::Error> {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        Sqlite::create_database(DB_URL).await?;
    }

    let pool = SqlitePool::connect(DB_URL).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS content (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    *DB_POOL.lock().await = Some(Arc::new(pool));
    Ok(())
}

pub async fn put_content(key: &Key, value: &str) -> Result<(), sqlx::Error> {
    let pool = DB_POOL.lock().await;
    let pool = pool.as_ref().unwrap();

    sqlx::query(
        r#"
        INSERT OR REPLACE INTO content (key, value)
        VALUES (?, ?)
        "#,
    )
    .bind(key.to_string())
    .bind(value)
    .execute(&**pool)
    .await?;

    Ok(())
}

pub async fn get_content(key: &Key) -> Result<Option<String>, sqlx::Error> {
    let pool = DB_POOL.lock().await;
    let pool = pool.as_ref().unwrap();

    let result = sqlx::query_scalar::<_, String>(
        r#"
        SELECT value FROM content WHERE key = ?
        "#,
    )
    .bind(key.to_string())
    .fetch_optional(&**pool)
    .await?;

    Ok(result)
}
