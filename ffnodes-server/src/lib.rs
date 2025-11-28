use actix_web::{App, HttpResponse, HttpServer, web};
use anyhow::Result;
use serde_json::json;
use std::env::set_current_dir;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use tracing_appender::rolling;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod clients;
mod configuration;
mod http_error;
mod jobs;
mod media_files;
mod templates;
mod jwt;
mod middleware;
mod path_security;
mod stats;

pub static DEBUG: bool = cfg!(debug_assertions);

pub async fn run() -> Result<()> {
    if DEBUG {
        set_current_dir("target/dev-env/server")?;
    }
    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("logs")?;

    // Set up rolling file appender (daily rotation + 500MB size limit)
    let file_appender = rolling::daily("logs", "ffnodes.log");

    // Create indicatif layer for progress bar integration
    let indicatif_layer = IndicatifLayer::new();

    // Set up multi-layer logging
    let console_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));
    let file_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"));

    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .pretty()
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_filter(console_filter),
        )
        .with(
            fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)
                .with_filter(file_filter),
        )
        .with(indicatif_layer)
        .init();

    serde_hash::hashids::SerdeHashOptions::new()
        .with_min_length(16)
        .build();

    let configuration = Arc::new(configuration::Configuration::load().await?);
    let port: u16 = configuration.port;
    let watch_directories = configuration.watch_directories.clone();
    let server_guid = configuration.server_guid.clone();

    info!("Server GUID: {}", server_guid);

    // Initialize database
    media_files::initialize().await?;

    // Open database pool
    let pool = media_files::media_file_db::open_pool().await?;

    // Initialize job queue
    let job_queue = Arc::new(jobs::JobQueue::new(pool.clone()));

    // Initialize client manager
    let client_manager = Arc::new(clients::ClientManager::new(pool.clone()));

    // Initialize progress broadcaster
    let _progress_broadcaster = media_files::progress::init_broadcaster(100);
    info!("Progress broadcaster initialized");

    // Create jobs for any existing unprocessed media files
    info!("Checking for unprocessed media files...");
    match job_queue.create_jobs_for_unprocessed_files().await {
        Ok(count) => {
            if count > 0 {
                info!("Created {} encoding jobs for existing unprocessed files", count);
            } else {
                info!("No unprocessed files found");
            }
        }
        Err(e) => {
            error!("Failed to create jobs for unprocessed files: {:#}", e);
        }
    }

    // Start job scheduler
    let job_scheduler = Arc::new(jobs::JobScheduler::new(
        Arc::clone(&job_queue),
        configuration.client_timeout_seconds as i64,
    ));
    job_scheduler.start();

    // Start file watcher
    let file_watcher = Arc::new(media_files::FileWatcher::new(
        Arc::clone(&configuration),
        pool.clone(),
        Arc::clone(&job_queue),
    ));
    if let Err(e) = file_watcher.start().await {
        warn!("Failed to start file watcher: {:#}", e);
    } else {
        info!("File watcher started");
    }

    // Initial scan
    tokio::spawn({
        let config = Arc::clone(&configuration);
        let job_queue_clone = Arc::clone(&job_queue);
        async move {
            if let Err(e) = media_files::Scanner::scan(watch_directories, config, job_queue_clone).await {
                error!("Media file scanner error: {}", e);
            }
        }
    });

    // Create WebSocket registry
    let ws_registry = api::websocket::create_ws_registry();

    // Clone for HttpServer closure
    let config_data = web::Data::new(Arc::clone(&configuration));
    let job_queue_data = web::Data::new(Arc::clone(&job_queue));
    let client_manager_data = web::Data::new(Arc::clone(&client_manager));
    let pool_data = web::Data::new(pool.clone());
    let ws_registry_data = web::Data::new(ws_registry.clone());

    let server = HttpServer::new(move || {
        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(config_data.clone())
            .app_data(job_queue_data.clone())
            .app_data(client_manager_data.clone())
            .app_data(pool_data.clone())
            .app_data(ws_registry_data.clone())
            .app_data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, _req| {
                        let error = json!({ "error": format!("{}", err) });
                        actix_web::error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().json(error),
                        )
                        .into()
                    }),
            )
            .service(
                web::scope("api")
                    // Authentication (no middleware required)
                    .route("/handshake", web::post().to(api::auth::handshake))
                    // Protected endpoints (JWT required)
                    .service(
                        web::scope("")
                            .wrap(actix_web::middleware::from_fn(middleware::jwt_auth::jwt_auth))
                            // Job endpoints
                            .route(
                                "/jobs/request/{client_id}",
                                web::post().to(api::jobs::request_job),
                            )
                            .route("/jobs/{job_id}/start", web::post().to(api::jobs::start_job))
                            .route(
                                "/jobs/{job_id}/progress",
                                web::post().to(api::jobs::update_progress),
                            )
                            .route(
                                "/jobs/{job_id}/complete",
                                web::post().to(api::jobs::complete_job),
                            )
                            .route("/jobs/{job_id}/fail", web::post().to(api::jobs::fail_job))
                            .route("/jobs/{job_id}/cancel", web::post().to(api::jobs::cancel_job))
                            .route("/jobs/active", web::get().to(api::jobs::get_active_jobs))
                            // File transfer
                            .route(
                                "/files/{job_id}/input",
                                web::get().to(api::files::download_input),
                            )
                            .route(
                                "/files/{job_id}/output",
                                web::post().to(api::files::upload_output),
                            )
                            // Heartbeat
                            .route(
                                "/heartbeat/{client_id}",
                                web::post().to(api::jobs::heartbeat),
                            )
                            // Monitoring
                            .route("/status", web::get().to(api::monitoring::get_status))
                            .route("/clients", web::get().to(api::monitoring::get_clients))
                            .route(
                                "/scan/progress",
                                web::get().to(api::monitoring::scan_progress),
                            )
                            // Statistics
                            .route(
                                "/stats/client/{client_id}/history",
                                web::get().to(api::stats::get_client_history),
                            )
                            .route(
                                "/stats/remote-progress",
                                web::get().to(api::stats::get_remote_progress),
                            )
                            .route(
                                "/stats/leaderboard",
                                web::get().to(api::stats::get_leaderboard),
                            )
                            // WebSocket
                            .route("/ws/progress", web::get().to(api::websocket::ws_progress))
                    )
            )
    })
    .workers(4)
    .bind(format!("0.0.0.0:{port}", port = port))?
    .run();

    info!(
        "Starting {} server at http://127.0.0.1:{}...",
        if DEBUG { "development" } else { "production" },
        port
    );

    let stop_result = server.await;
    debug!("Server stopped");

    Ok(stop_result?)
}
