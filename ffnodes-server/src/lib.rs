use actix_web::{App, HttpResponse, HttpServer, middleware, web};
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
    let console_filter = if DEBUG {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    let file_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

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
        async move {
            if let Err(e) = media_files::Scanner::scan(watch_directories, config).await {
                error!("Media file scanner error: {}", e);
            }
        }
    });

    // Clone for HttpServer closure
    let config_data = web::Data::new(Arc::clone(&configuration));
    let job_queue_data = web::Data::new(Arc::clone(&job_queue));
    let client_manager_data = web::Data::new(Arc::clone(&client_manager));

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(config_data.clone())
            .app_data(job_queue_data.clone())
            .app_data(client_manager_data.clone())
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
                    // Authentication
                    .route("/handshake", web::post().to(api::auth::handshake))
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
                    .route("/jobs/active", web::get().to(api::jobs::get_active_jobs))
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
                    // WebSocket
                    .route("/ws/progress", web::get().to(api::websocket::ws_progress)),
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
