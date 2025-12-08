// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use dirs::data_local_dir;
use log::{error, info};
use serde::{Deserialize, Serialize};
use sqlx::{query_as, FromRow, SqlitePool};
use std::fs::{copy, create_dir, read_dir};
use std::io::ErrorKind;
use std::path::PathBuf;
use tauri::State;
use time::{macros::format_description, OffsetDateTime};

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tab_id: Option<i64>,
    pub parent_id: Option<i64>,
    pub order_id: Option<i64>,
    pub note_type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Tab {
    pub id: i64,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn get_notes(
    pool: State<'_, SqlitePool>,
    tab_id: Option<i64>,
) -> Result<Vec<Note>, String> {
    let notes = query_as::<_, Note>(
        r#"
        SELECT * FROM notes
        WHERE tab_id IS NOT DISTINCT FROM ?
        ORDER BY order_id ASC
        "#,
    )
    .bind(tab_id)
    .fetch_all(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch notes (tab_id = {:?}): {:#}", tab_id, e);
        "Failed to load notes. Please try again.".to_string()
    })?;

    Ok(notes)
}

#[tauri::command]
pub async fn create_note(
    pool: State<'_, SqlitePool>,
    title: String,
    content: String,
    tab_id: Option<i64>,
    parent_id: Option<i64>,
    note_type: String,
) -> Result<Note, String> {
    let row: Option<(Option<i64>,)> = sqlx::query_as(
        r#"
        SELECT MAX(order_id) FROM notes WHERE tab_id IS NOT DISTINCT FROM ? AND parent_id IS NOT DISTINCT FROM ?
        "#,
    )
    .bind(tab_id)
    .bind(parent_id)
    .fetch_optional(&*pool)
    .await
    .map_err(|e| {
        error!("Database error: {:#}", e);
        e.to_string()
    })?;

    let max_order = row.and_then(|r| r.0).unwrap_or(0);
    let new_order = max_order + 1;

    let note = query_as::<_, Note>(
        r#"
        INSERT INTO notes (title, content, tab_id, parent_id, order_id, note_type)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id, title, content, tab_id, parent_id, order_id, note_type, created_at, updated_at
        "#,
    )
    .bind(&title)
    .bind(content)
    .bind(tab_id)
    .bind(parent_id)
    .bind(new_order)
    .bind(note_type)
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to create note {}: {:#}", title, e);
        "Failed to create note. Please try again".to_string()
    })?;

    Ok(note)
}

#[tauri::command]
pub async fn update_note(
    pool: State<'_, SqlitePool>,
    id: i64,
    title: String,
    content: String,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE notes
        SET title = ?,
            content = ?,
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(title)
    .bind(content)
    .bind(id)
    .execute(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to update note {}: {:#}", id, e);
        "Failed to update note. Please try again".to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub async fn delete_note(pool: State<'_, SqlitePool>, id: i64) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to delete note {}: {:#}", id, e);
            "Failed to delete note. Please try again".to_string()
        })?;

    if result.rows_affected() == 0 {
        return Err(format!("Note with id {id} not found"));
    }

    Ok(())
}

#[tauri::command]
pub async fn get_tabs(pool: State<'_, SqlitePool>) -> Result<Vec<Tab>, String> {
    let tabs = query_as::<_, Tab>(
        r#"
        SELECT * FROM tabs
        ORDER BY id ASC
        "#,
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch tabs: {:#}", e);
        "Failed to fetch tabs. Please try again".to_string()
    })?;

    Ok(tabs)
}

#[tauri::command]
pub async fn create_tab(pool: State<'_, SqlitePool>, name: String) -> Result<Tab, String> {
    let tab = query_as::<_, Tab>(
        r#"
        INSERT INTO tabs (name)
        VALUES (?)
        RETURNING id, name, created_at, updated_at
        "#,
    )
    .bind(&name)
    .fetch_one(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to create tab {}: {:#}", name, e);
        "Failed to create tab. Please try again".to_string()
    })?;

    Ok(tab)
}

#[tauri::command]
pub async fn update_tab(pool: State<'_, SqlitePool>, id: i64, name: String) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE tabs
        SET name = ?,
            updated_at = datetime('now')
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(id)
    .execute(&*pool)
    .await
    .map_err(|e| {
        error!("Failed to update tab {}: {:#}", id, e);
        "Failed to update tab. Please try again".to_string()
    })?;

    Ok(())
}

#[tauri::command]
pub async fn delete_tab(pool: State<'_, SqlitePool>, id: i64) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM tabs WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to delete tab {}: {:#}", id, e);
            "Failed to delete tab. Please try again".to_string()
        })?;

    if result.rows_affected() == 0 {
        return Err(format!("Tab with id {id} not found"));
    }

    Ok(())
}

#[tauri::command]
pub async fn backup_database() -> Result<(), String> {
    let local_data_dir: PathBuf = data_local_dir().ok_or("Failed to get local data directory")?;
    let identifier = "com.stenberg.focusboard";
    let app_local_data_dir = local_data_dir.join(identifier);

    let database_path: PathBuf = app_local_data_dir.join("database");
    let backup_base_path: PathBuf = app_local_data_dir.join("database_backups");

    match create_dir(&backup_base_path) {
        Ok(()) => {
            info!("Database backup path created: {:?}", backup_base_path);
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            info!("Database backup path already exists {:?}", backup_base_path);
        }
        Err(e) => {
            error!(
                "Failed to create database backup path {:?}: {:#}",
                backup_base_path, e
            );
        }
    }

    let now = OffsetDateTime::now_local().map_err(|e| {
        error!("Failed to get local time: {:#}", e);
        e.to_string()
    })?;

    let timestamp = now
        .format(&format_description!(
            "[year]-[month]-[day]_T[hour]H-[minute]M-[second]S"
        ))
        .map_err(|e| {
            error!("Failed to format local time: {:#}", e);
            e.to_string()
        })?;

    let backup_path: PathBuf = backup_base_path.join(format!("database-backup_{}", timestamp));

    match create_dir(&backup_path) {
        Ok(()) => {
            info!("Backup directory successfully created");
        }
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            info!("Backup directory already exists {:?}", backup_path);
        }
        Err(e) => {
            error!("Failed to create backup directory: {:#}", e);
        }
    }

    let entries = read_dir(&database_path).map_err(|e| {
        error!("Failed to read entries from {:?}: {:#}", database_path, e);
        "Failed to read entries from database directory.".to_string()
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            error!("Failed to read directory entry: {:#}", e);
            "Failed to read database directory entry.".to_string()
        })?;

        let src_path = entry.path();
        let dest_path = backup_path.join(entry.file_name());

        copy(&src_path, &dest_path).map_err(|e| {
            error!("Failed to copy {:?} to {:?}: {:#}", src_path, dest_path, e);
            "Failed to copy database to destination.".to_string()
        })?;
    }
    info!("Database backup successfully completed");

    Ok(())
}

#[tauri::command]
pub async fn reorder_notes(
    pool: State<'_, SqlitePool>,
    note_ids: Vec<i64>
) -> Result<(), String> {
    let len = note_ids.len() as i64;
    if len == 0 {
        return Ok(());
    }

    let mut transaction = pool.begin().await.map_err(|e| {
        error!("Failed to start transaction: {:#}", e);
        "Failed to start transaction".to_string()
    })?;

    for (index, &id) in note_ids.iter().enumerate() {
        let order_id = (index + 1) as i64;

        sqlx::query("UPDATE notes SET order_id = ? WHERE id = ?")
            .bind(order_id)
            .bind(id)
            .execute(&mut *transaction)
            .await
            .map_err(|e| {
                error!("Failed to reorder note {}: {:#}", id, e);
                "Failed to reorder notes".to_string()
            })?;
    }

    transaction.commit().await.map_err(|e| {
        error!("Failed to commit transaction: {:#}", e);
        "Failed to commit transaction".to_string()
    })?;

    Ok(())
}
