// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, FromRow, query_as};
use tauri::State;
use log::{error};

#[derive(FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub tab_id: Option<i64>,
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
        ORDER BY updated_at DESC
        "#
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
) -> Result<Note, String> {

    let note = query_as::<_, Note>(
        r#"
        INSERT INTO notes (title, content, tab_id)
        VALUES (?, ?, ?)
        RETURNING id, title, content, tab_id, created_at, updated_at
        "#
    )
    .bind(&title)
    .bind(content)
    .bind(tab_id)
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
        "#
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
pub async fn delete_note(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM notes WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to delete note {}: {:#}", id, e);
            "Failed to delete note. Please try again".to_string()
        })?;

    if result.rows_affected() == 0 {
        return Err(format!("Note with id {id} not found"))
    }

    Ok(())
}

#[tauri::command]
pub async fn get_tabs(
    pool: State<'_, SqlitePool>,
) -> Result<Vec<Tab>, String> {
    let tabs = query_as::<_, Tab>(
        r#"
        SELECT * FROM tabs
        ORDER BY id ASC
        "#
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
pub async fn create_tab(
    pool: State<'_, SqlitePool>,
    name: String,
) -> Result<Tab, String> {

    let tab = query_as::<_, Tab>(
        r#"
        INSERT INTO tabs (name)
        VALUES (?)
        RETURNING id, name, created_at, updated_at
        "#
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
pub async fn update_tab(
    pool: State<'_, SqlitePool>,
    id: i64,
    name: String,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE tabs
        SET name = ?,
            updated_at = datetime('now')
        WHERE id = ?
        "#
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
pub async fn delete_tab(
    pool: State<'_, SqlitePool>,
    id: i64,
) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM tabs WHERE id = ?")
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            error!("Failed to delete tab {}: {:#}", id, e);
            "Failed to delete tab. Please try again".to_string()
        })?;

    if result.rows_affected() == 0 {
        return Err(format!("Tab with id {id} not found"))
    }

    Ok(())
}
