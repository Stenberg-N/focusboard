// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use tauri::State;

#[derive(Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tab_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<sqlx::sqlite::SqliteRow> for Note {
    fn from(row: sqlx::sqlite::SqliteRow) -> Self {
        Note {
            id: row.try_get("id").unwrap(),
            title: row.try_get("title").unwrap(),
            content: row.try_get("content").unwrap(),
            tab_id: row.try_get("tab_id").ok(),
            created_at: row.try_get("created_at").unwrap(),
            updated_at: row.try_get("updated_at").unwrap(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Tab {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<sqlx::sqlite::SqliteRow> for Tab {
    fn from(row: sqlx::sqlite::SqliteRow) -> Self {
        Tab {
            id: row.try_get("id").unwrap(),
            name: row.try_get("name").unwrap(),
            created_at: row.try_get("created_at").unwrap(),
            updated_at: row.try_get("updated_at").unwrap(),
        }
    }
}

#[tauri::command]
pub async fn get_notes(
    pool: State<'_, SqlitePool>,
    tab_id: Option<i64>,
) -> Result<Vec<Note>, String> {
    let query = if let Some(tid) = tab_id {
        sqlx::query("SELECT * FROM notes WHERE tab_id = ? ORDER BY updated_at DESC").bind(tid)
    } else {
        sqlx::query("SELECT * FROM notes WHERE tab_id IS NULL ORDER BY updated_at DESC")
    };

    let rows = query
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let notes = rows.into_iter().map(Note::from).collect();
    Ok(notes)
}

#[tauri::command]
pub async fn create_note(
    pool: State<'_, SqlitePool>,
    title: &str,
    content: &str,
    tab_id: Option<i64>,
) -> Result<Note, String> {
    let query = sqlx::query(
        "INSERT INTO notes (title, content, tab_id) VALUES (?, ?, ?)
        RETURNING id, title, content, tab_id, created_at, updated_at"
    )
    .bind(title)
    .bind(content)
    .bind(tab_id);

    let row = query
        .fetch_one(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Note::from(row))
}

#[tauri::command]
pub async fn update_note(
    pool: State<'_, SqlitePool>,
    id: i64,
    title: &str,
    content: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE notes SET title = ?, content = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(title)
    .bind(content)
    .bind(id)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_note(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn get_tabs(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Tab>, String> {
    let rows = sqlx::query("SELECT * FROM tabs ORDER BY id ASC")
        .fetch_all(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    let tabs = rows.into_iter().map(Tab::from).collect();
    Ok(tabs)
}

#[tauri::command]
pub async fn create_tab(
    pool: State<'_, SqlitePool>,
    name: &str,
) -> Result<Tab, String> {
    let row = sqlx::query(
        "INSERT INTO tabs (name) VALUES (?)
        RETURNING id, name, created_at, updated_at"
    )
    .bind(name)
    .fetch_one(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Tab::from(row))
}

#[tauri::command]
pub async fn update_tab(
    pool: State<'_, SqlitePool>,
    id: i64,
    name: &str,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE tabs SET name = ?, updated_at = datetime('now') WHERE id = ?"
    )
    .bind(name)
    .bind(id)
    .execute(&*pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_tab(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    sqlx::query("DELETE FROM notes WHERE tab_id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM tabs WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}