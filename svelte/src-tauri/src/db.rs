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
            order_id INTEGER,
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
            parent_id INTEGER,
            order_id INTEGER,
            note_type TEXT,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (tab_id) REFERENCES tabs(id) ON DELETE CASCADE,
            FOREIGN KEY (parent_id) REFERENCES notes(id) ON DELETE CASCADE
        )",
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS timers (
            id INTEGER PRIMARY KEY,
            initial_duration INTEGER,
            duration INTEGER,
            message TEXT
        )"
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id INTEGER PRIMARY KEY,
            event_date TEXT NOT NULL,
            event_name TEXT,
            event_start INTEGER,
            event_end INTEGER
        )"
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_notes_tab_parent_order
        ON notes(tab_id, parent_id, order_id)"
    ).execute(&mut *conn).await?;

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_notes_parent_order
        ON notes(parent_id, order_id)"
    ).execute(&mut *conn).await?;

    Ok(db)
}
