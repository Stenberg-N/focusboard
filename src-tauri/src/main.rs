// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;

use colored::*;
use log::{error, info, warn};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{async_runtime, Emitter, Manager};
use tauri_plugin_log::{Target, TargetKind};

#[async_std::main]
async fn main() {
    let context = tauri::generate_context!();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .level(log::LevelFilter::Info)
                .format(|out, message, record| {
                    let level = match record.level() {
                        log::Level::Error => "ERROR".red(),
                        log::Level::Warn => "WARN".yellow(),
                        log::Level::Info => "INFO".green(),
                        log::Level::Debug => "DEBUG".purple(),
                        log::Level::Trace => "TRACE".blue(),
                    };
                    out.finish(format_args!("{level} | {}", message))
                })
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .max_file_size(50000)
                .filter(|md| !md.target().starts_with("tao::platform_impl"))
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let base_dir: PathBuf = app.path().app_local_data_dir()?.into();
            let data_dir = base_dir.join("database");

            info!("Attempting to create database directory: {:?}", data_dir);
            if let Err(e) = fs::create_dir_all(&data_dir) {
                if e.kind() != ErrorKind::AlreadyExists {
                    error!(
                        "Failed to create database directory {:?}: {:#}",
                        data_dir, e
                    );
                }
            } else {
                info!("Database directory already exists: {:?}", data_dir);
            }
            info!("Database directory ready: {:?}", data_dir);

            let db_path = data_dir.join("data.db");
            let db_path_str = db_path.to_string_lossy().replace("\\", "/");
            let db_url = format!("sqlite://{}?mode=rwc", db_path_str);

            info!("DB URL: {}", db_url);

            let pool = async_runtime::block_on(db::init_db(&db_url)).map_err(|e| {
                error!("Failed to initialize database at {:?}: {:#}", db_url, e);
                e
            })?;

            let pool_cleanup_background = pool.clone();
            let pool_cleanup_close = pool.clone();

            tauri::async_runtime::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(600));
                loop {
                    interval.tick().await;
                    let _ = sqlx::query("PRAGMA optimize")
                        .execute(&pool_cleanup_background)
                        .await
                        .map_err(|e| {
                            warn!("Periodic database optimization failed: {:#}", e);
                            e
                        });
                }
            });

            let is_closing = Arc::new(Mutex::new(false));

            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                let is_closing = is_closing.clone();

                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();

                        let mut closing = is_closing.lock().unwrap();
                        if *closing {
                            return;
                        }
                        *closing = true;
                        drop(closing);

                        let _ = window_clone.emit("app-closing", ());

                        let win = window_clone.clone();
                        let pool = pool_cleanup_close.clone();

                        tauri::async_runtime::spawn(async move {
                            let _ = sqlx::query("PRAGMA optimize;")
                                .execute(&pool)
                                .await
                                .map_err(|e| {
                                    warn!("Failed to optimize database: {:#}", e);
                                    e
                                });
                            let _ = sqlx::query("PRAGMA incremental_vacuum(0);")
                                .execute(&pool)
                                .await
                                .map_err(|e| {
                                    warn!("Failed to incrementally vacuum database: {:#}", e);
                                    e
                                });
                            let _ = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE);")
                                .execute(&pool)
                                .await
                                .map_err(|e| {
                                    warn!("Failed to flush WAL: {:#}", e);
                                    e
                                });

                            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                            let _ = win.close();
                            win.app_handle().exit(0);
                        });
                    }
                });
            }

            app.manage(pool);

            info!("App setup complete");
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
            commands::delete_tab,
            commands::backup_database,
            commands::reorder_notes,
        ])
        .run(context)
        .expect("Error while running tauri application");
}
