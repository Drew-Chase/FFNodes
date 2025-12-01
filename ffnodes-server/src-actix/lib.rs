use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, web};
use anyhow::Result;
use serde_json::json;
use std::env::set_current_dir;
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use tracing_appender::rolling;
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use vite_actix::proxy_vite_options::ProxyViteOptions;
use vite_actix::start_vite_server;
use crate::asset_endpoint::AssetsAppConfig;

mod api;
mod asset_endpoint;
mod clients;
mod configuration;
mod http_error;
mod job_actor;
mod jobs;
mod media_files;
mod templates;
mod jwt;
mod middleware;
mod path_security;
mod stats;

pub static DEBUG: bool = cfg!(debug_assertions);

/// Set up logging with custom rotation
async fn setup_logging() -> Result<()> {
    use std::path::Path;
    use std::fs;

    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("logs")?;

    let latest_log = Path::new("logs/ffnodes-server.latest.log");

    // Archive previous log file if it exists
    if latest_log.exists() {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let mut archived_log = format!("logs/ffnodes-server.{}.log", today);

        // Handle multiple runs on same day (append counter)
        let mut counter = 1;
        while Path::new(&archived_log).exists() {
            archived_log = format!("logs/ffnodes-server.{}.{}.log", today, counter);
            counter += 1;
        }

        fs::rename(latest_log, &archived_log)?;
        info!("Rotated previous log to: {}", archived_log);
    }

    // Create file appender for latest.log (never rotate during runtime)
    let file_appender = rolling::never("logs", "ffnodes-server.latest.log");

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

    Ok(())
}

pub async fn run() -> Result<()> {
    #[cfg(debug_assertions)]
    {
        ProxyViteOptions::new().port(5173).working_directory(std::path::Path::new("ffnodes-server").canonicalize()?.to_string_lossy().as_ref()).disable_logging().build()?;
        std::thread::spawn(|| {
            loop {
                info!("Starting Vite server in development mode...");
                let status = start_vite_server().expect("Failed to start vite server").wait().expect("Vite server crashed!");
                if !status.success() {
                    error!("The vite server has crashed!");
                } else {
                    break;
                }
            }
        });
        set_current_dir("target/dev-env/server")?;
    }

    // Set up logging with custom rotation
    setup_logging().await?;

    // Initialize serde_hash
    serde_hash::hashids::SerdeHashOptions::new()
        .with_min_length(16)
        .build();

    // Load configuration
    let configuration = Arc::new(configuration::Configuration::load().await?);
    let port: u16 = configuration.port;

    info!("Server GUID: {}", configuration.server_guid);

    // Initialize progress broadcaster
    let _progress_broadcaster = media_files::progress::init_broadcaster(100);
    info!("Progress broadcaster initialized");

    // Send test progress for debugging SSE
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        media_files::progress::send_progress(media_files::progress::ScanProgress {
            total_files: 10,
            completed_files: 3,
            current_file: Some("test.mp4".to_string()),
            operation: "Testing SSE".to_string(),
        });
        info!("Sent test SSE progress");
    });

    // Create WebSocket registry
    let ws_registry = api::websocket::create_ws_registry();

    // Initialize database (for stats queries and actor)
    info!("Initializing database...");
    media_files::initialize().await?;
    let pool = media_files::media_file_db::open_pool().await?;
    info!("Database initialized");

    // Create actor command channel (buffer: 1000 commands)
    let (actor_tx, actor_rx) = tokio::sync::mpsc::channel(1000);
    let actor_handle = job_actor::JobActorHandle::new(actor_tx.clone());

    // Spawn job management actor (pass pool for faster initialization)
    let actor = job_actor::JobActor::new(Arc::clone(&configuration), Some(pool.clone()));
    tokio::spawn(async move {
        actor.run(actor_rx).await;
    });

    info!("Job management actor spawned, HTTP server starting immediately");

    // Optional: Monitor initialization progress
    tokio::spawn({
        let handle = actor_handle.clone();
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                match handle.get_initialization_status().await {
                    job_actor::InitializationStatus::Complete => {
                        info!("Job management actor fully initialized");
                        break;
                    }
                    job_actor::InitializationStatus::Failed { error } => {
                        error!("Actor initialization failed: {}", error);
                        break;
                    }
                    job_actor::InitializationStatus::InProgress { stage } => {
                        info!("Initialization: {}", stage);
                    }
                    _ => {}
                }
            }
        }
    });

    // Prepare HTTP server data
    let config_data = web::Data::new(Arc::clone(&configuration));
    let actor_data = web::Data::new(actor_handle.clone());
    let pool_data = web::Data::new(pool.clone());
    let ws_registry_data = web::Data::new(ws_registry.clone());


    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600)
            )
            .wrap(actix_web::middleware::Logger::default())
            .app_data(config_data.clone())
            .app_data(actor_data.clone())
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
                            .configure(api::ping::configure)
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
            .configure_frontend_routes()
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

    // Send shutdown command to actor
    info!("Shutting down job management actor...");
    let (tx, rx) = tokio::sync::oneshot::channel();
    if let Err(e) = actor_tx.send(job_actor::ActorCommand::Shutdown { respond_to: tx }).await {
        warn!("Failed to send shutdown command to actor: {}", e);
    } else if let Err(e) = tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
        warn!("Actor shutdown timeout: {}", e);
    }

    Ok(stop_result?)
}
