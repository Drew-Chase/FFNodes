use anyhow::Result;
use std::path::PathBuf;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize file and console logging
pub fn init() -> Result<PathBuf> {
    let log_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("FFNodes")
        .join("logs");

    std::fs::create_dir_all(&log_dir)?;

    // Create rolling file appender (daily rotation, keeps last 7 days)
    let file_appender = rolling::daily(&log_dir, "ffnodes-client");

    // Create formatting layer for file
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_line_number(true)
        .with_file(true);

    // Create formatting layer for console
    let console_layer = fmt::layer()
        .with_target(true)
        .with_line_number(true);

    // Set up env filter (defaults to INFO, can be overridden with RUST_LOG env var)
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,ffnodes_client=debug"));

    // Initialize subscriber with both console and file output
    tracing_subscriber::registry()
        .with(env_filter)
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
