use actix_web::{App, HttpResponse, HttpServer, middleware, web};
use anyhow::Result;
use log::*;
use serde_json::json;
use std::env::set_current_dir;
use std::sync::Arc;

mod configuration;
mod http_error;
mod media_files;

pub static DEBUG: bool = cfg!(debug_assertions);

pub async fn run() -> Result<()> {
    pretty_env_logger::env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    serde_hash::hashids::SerdeHashOptions::new()
        .with_min_length(16)
        .build();

    if DEBUG {
        info!("Debug mode enabled");
        set_current_dir("target/dev-env/server")?;
    }

    let configuration = Arc::new(configuration::Configuration::load().await?);
    let port: u16 = configuration.port;
    let watch_directories = configuration.watch_directories.clone();

    media_files::initialize().await?;

    tokio::spawn({
        let config = Arc::clone(&configuration);
        async move {
            if let Err(e) = media_files::Scanner::scan(watch_directories, config).await {
                error!("Media file scanner error: {}", e);
            }
        }
    });

    let server = HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
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
            .service(web::scope("api"))
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
