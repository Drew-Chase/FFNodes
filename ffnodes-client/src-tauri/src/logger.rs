use anyhow::Result;
use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize file and console logging with custom rotation
pub fn init() -> Result<PathBuf> {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("FFNodes")
        .join("logs");

    std::fs::create_dir_all(&log_dir)?;

    let latest_log = log_dir.join("ffnodes-client.latest.log");

    // Archive previous log file if it exists
    if latest_log.exists() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut archived_log = log_dir.join(format!("ffnodes-client.{}.log", today));

        // Handle multiple runs on same day (append counter)
        let mut counter = 1;
        while archived_log.exists() {
            archived_log = log_dir.join(format!("ffnodes-client.{}.{}.log", today, counter));
            counter += 1;
        }

        std::fs::rename(&latest_log, &archived_log)?;
        // Log after tracing is initialized
    }

    // Create file appender for latest.log (never rotate during runtime)
    let file_appender = rolling::never(&log_dir, "ffnodes-client.latest.log");

    // Create formatting layer for file with trace level
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_line_number(true)
        .with_file(true)
        .with_filter(EnvFilter::new("trace"));

    // Create formatting layer for console with trace level
    let console_layer = fmt::layer()
        .with_target(true)
        .with_line_number(true)
        .with_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace")));

    // Initialize subscriber with both console and file output
    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("Logging initialized");
    tracing::info!("Log directory: {}", log_dir.display());

    Ok(log_dir)
}

/// Log a frontend message (called from frontend via Tauri command)
pub fn log_frontend(level: String, message: String, source: Option<String>) {
    let source = source.unwrap_or_else(|| "frontend".to_string());

    match level.to_lowercase().as_str() {
        "error" => tracing::error!(target: "frontend", source = %source, "{}", message),
        "warn" => tracing::warn!(target: "frontend", source = %source, "{}", message),
        "info" => tracing::info!(target: "frontend", source = %source, "{}", message),
        "debug" => tracing::debug!(target: "frontend", source = %source, "{}", message),
        "trace" => tracing::trace!(target: "frontend", source = %source, "{}", message),
        _ => tracing::info!(target: "frontend", source = %source, "{}", message),
    }
}
