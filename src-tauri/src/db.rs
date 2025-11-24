use sqlx::SqlitePool;

pub async fn init_db(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let db = SqlitePool::connect(db_url).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS tabs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now'))
        )"
    )
    .execute(&db)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS notes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            content TEXT,
            tab_id INTEGER,
            created_at TEXT DEFAULT (datetime('now')),
            updated_at TEXT DEFAULT (datetime('now')),
            FOREIGN KEY (tab_id) REFERENCES tabs(id)
        )"
    )
    .execute(&db)
    .await?;

    Ok(db)
}