use actix_web::{App, HttpResponse, HttpServer, web};
use actix_files as files;
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

    // Use consistent formatting for both console and file
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(true)
                .with_line_number(true)
                .with_writer(indicatif_layer.get_stderr_writer())
                .with_filter(console_filter),
        )
        .with(
            fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_thread_names(false)
                .with_file(true)
                .with_line_number(true)
                .with_writer(file_appender)
                .with_ansi(false)
                .with_filter(file_filter),
        )
        .with(indicatif_layer)
        .init();

    // Set up panic hook to log panics
    std::panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            *s
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.as_str()
        } else {
            "Box<dyn Any>"
        };

        let location = if let Some(location) = panic_info.location() {
            format!(" at {}:{}:{}", location.file(), location.line(), location.column())
        } else {
            String::new()
        };

        error!("thread panicked with message: {}{}", message, location);
    }));

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
                    .configure(api::auth::configure)
                    // Public endpoints (no JWT required) for dashboard
                    .service(
                        web::scope("public")
                            .configure(api::monitoring::configure_public)
                            .configure(api::stats::configure_public)
                            .configure(api::jobs::configure_public)
                    )
                    // Protected endpoints (JWT required)
                    .service(
                        web::scope("")
                            .wrap(actix_web::middleware::from_fn(middleware::jwt_auth::jwt_auth))
                            .configure(api::jobs::configure)
                            .configure(api::files::configure)
                            .configure(api::monitoring::configure)
                            .configure(api::stats::configure)
                            .configure(api::websocket::configure)
                            .default_service(web::to(|| async {
                                HttpResponse::NotFound().json(json!({
                                    "error": "API endpoint not found".to_string(),
                                }))
                            }))
                    )
                    .default_service(web::to(|| async {
                        HttpResponse::NotFound().json(json!({
                            "error": "API endpoint not found".to_string(),
                        }))
                    }))
            )
            // Static file serving for dashboard frontend (place after API routes)
            .service(
                files::Files::new("/", "target/wwwroot")
                    .index_file("index.html")
                    .use_last_modified(true)
                    .default_handler(web::to(|req: actix_web::HttpRequest| async move {
                        match files::NamedFile::open_async("target/wwwroot/index.html").await {
                            Ok(file) => file.into_response(&req),
                            Err(_) => HttpResponse::NotFound().body("Frontend not built. Run 'pnpm build-frontend' in ffnodes-server directory.")
                        }
                    }))
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
