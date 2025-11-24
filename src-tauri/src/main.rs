// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod commands;

use std::fs;
use std::path::PathBuf;
use tauri::async_runtime;
use tauri::Manager;

#[async_std::main]
async fn main() {
    let context = tauri::generate_context!();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let mut data_dir: PathBuf = app.path().app_local_data_dir()?.into();
            data_dir.push("notes_app");

            eprintln!("Attempting to create data dir: {:?}", data_dir);

            fs::create_dir_all(&data_dir).map_err(|e| format!("Failed to create data dir {:?}: {}", data_dir, e))?;

            let db_path = data_dir.join("data.db");
            let db_path_str = db_path.to_string_lossy().replace("\\", "/");
            let db_url = format!("sqlite://{}?mode=rwc", db_path_str);

            eprintln!("DB URL: {}", db_url);

            let pool = async_runtime::block_on(db::init_db(&db_url))
                .map_err(|e| format!("Failed to initialize database at {}: {}", db_url, e))?;

            app.manage(pool);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_notes,
            commands::create_note,
            commands::update_note,
            commands::delete_note,
            commands::get_tabs,
            commands::create_tab,
            commands::update_tab,
            commands::delete_tab
        ])
        .run(context)
        .expect("Error while running tauri application");
}