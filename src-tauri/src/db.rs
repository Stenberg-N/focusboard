use sqlx::{sqlite::SqlitePoolOptions, Executor, SqlitePool};

pub async fn init_db(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let db = SqlitePoolOptions::new()
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute(sqlx::query(
                    "PRAGMA journal_mode = WAL;\
                    PRAGMA foreign_keys = ON;\
                    PRAGMA auto_vacuum = INCREMENTAL;\
                    PRAGMA optimize;",
                ))
                .await?;

                Ok(())
            })
        })
        .connect(db_url)
        .await?;

    let mut conn = db.acquire().await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tabs (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        )",
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY,
            title TEXT NOT NULL,
            content TEXT,
            tab_id INTEGER,
            note_type TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (tab_id) REFERENCES tabs(id) ON DELETE CASCADE
        )",
    )
    .execute(&mut *conn)
    .await?;

    Ok(db)
}
