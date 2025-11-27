mod api;
mod commands;
mod config;
mod encoder;
mod gpu;
mod job_manager;
mod logger;
mod oauth;

use commands::*;
use job_manager::JobManager;
use oauth::*;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_oauth::init())
        .setup(|app| {
            // Initialize logging
            match logger::init() {
                Ok(log_dir) => {
                    tracing::info!("FFNodes Client starting...");
                    tracing::info!("Logs will be written to: {}", log_dir.display());
                }
                Err(e) => {
                    eprintln!("Failed to initialize logging: {:#}", e);
                }
            }

            // Create and store JobManager
            tracing::info!("Creating JobManager...");
            let job_manager = tauri::async_runtime::block_on(async {
                JobManager::new(app.handle().clone()).await
            })
            .expect("Failed to create JobManager");
            app.manage(Arc::new(Mutex::new(job_manager)));
            tracing::info!("JobManager created successfully");

            // Setup system tray
            let _tray = tauri::tray::TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("FFNodes Client")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "hide" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                    "pause" => {
                        let job_manager = app.state::<Arc<Mutex<JobManager>>>().inner().clone();
                        tauri::async_runtime::spawn(async move {
                            let manager = job_manager.lock().await;
                            let _ = manager.pause().await;
                        });
                    }
                    "resume" => {
                        let job_manager = app.state::<Arc<Mutex<JobManager>>>().inner().clone();
                        tauri::async_runtime::spawn(async move {
                            let manager = job_manager.lock().await;
                            let _ = manager.resume().await;
                        });
                    }
                    "quit" => {
                        if let Some(window) = app.get_webview_window("main")
                            && let Ok(visible) = window.is_visible()
                            && visible
                            && let Err(e) = app.save_window_state(StateFlags::all())
                        {
                            tracing::error!("Failed to save window state: {}", e);
                        }
                        app.exit(0);
                    }
                    _ => {}
                })
                .menu(
                    &tauri::menu::MenuBuilder::new(app)
                        .item(
                            &tauri::menu::MenuItemBuilder::new("Show")
                                .id("show")
                                .build(app)?,
                        )
                        .item(
                            &tauri::menu::MenuItemBuilder::new("Hide")
                                .id("hide")
                                .build(app)?,
                        )
                        .separator()
                        .item(
                            &tauri::menu::MenuItemBuilder::new("Pause")
                                .id("pause")
                                .build(app)?,
                        )
                        .item(
                            &tauri::menu::MenuItemBuilder::new("Resume")
                                .id("resume")
                                .build(app)?,
                        )
                        .separator()
                        .item(
                            &tauri::menu::MenuItemBuilder::new("Quit")
                                .id("quit")
                                .build(app)?,
                        )
                        .build()?,
                )
                .build(app)?;

            // Handle window close event - minimize to tray instead
            if let Some(window) = app.get_webview_window("main") {
                if let Err(e) = window.restore_state(StateFlags::all()) {
                    tracing::error!("Failed to window restore state: {}", e);
                }
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        let handle = window_clone.app_handle();
                        if let Err(e) = handle.save_window_state(StateFlags::all()) {
                            tracing::error!("Failed to save window state: {}", e);
                        }

                        // Prevent window from closing, hide it instead
                        api.prevent_close();
                        let _ = window_clone.hide();
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_config,
            save_config,
            test_connection,
            get_gpu_info,
            extract_frame,
            get_active_jobs,
            start_job_processing,
            stop_job_processing,
            pause_job_processing,
            resume_job_processing,
            get_job_manager_state,
            log_frontend,
            start_oauth_flow,
            refresh_oauth_token
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
