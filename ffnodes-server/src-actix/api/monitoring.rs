use crate::http_error::Error;
use crate::job_actor::{ActorError, JobActorHandle};
use crate::media_files::progress;
use actix_web::{get, web, HttpResponse, HttpRequest};
use tracing::{debug, warn, error};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use serde_json::json;

/// Convert actor error to HTTP error
fn map_actor_error(err: ActorError) -> Error {
    match err {
        ActorError::NotInitialized => {
            Error::service_unavailable("Server is still initializing, please try again")
        }
        ActorError::DatabaseError(msg) => {
            error!("Database error: {}", msg);
            Error::internal_server_error(&msg)
        }
        ActorError::NotFound => {
            Error::not_found("Resource not found")
        }
        ActorError::InvalidState(msg) => {
            error!("Invalid state: {}", msg);
            Error::internal_server_error(&msg)
        }
    }
}

/// System status response
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub total_media_files: i64,
    pub pending_jobs: i64,
    pub active_jobs: i64,
    pub connected_clients: usize,
}

/// Get overall system status
#[get("/status")]
pub async fn get_status(
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    debug!("Getting system status");

    let pending_jobs = actor.get_pending_count().await.map_err(|e| {
        warn!("Error getting pending jobs count: {:#}", e);
        map_actor_error(e)
    })?;

    let active_jobs = actor.get_active_count().await.map_err(|e| {
        warn!("Error getting active jobs count: {:#}", e);
        map_actor_error(e)
    })?;

    let clients = actor.get_connected_clients().await.map_err(|e| {
        warn!("Error getting connected clients: {:#}", e);
        map_actor_error(e)
    })?;

    let status = SystemStatus {
        total_media_files: 0, // TODO: Get from database
        pending_jobs,
        active_jobs,
        connected_clients: clients.len(),
    };

    Ok(HttpResponse::Ok().json(status))
}

/// Get all connected clients with their statuses
#[get("/clients")]
pub async fn get_clients(
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    debug!("Getting all clients");

    let clients = actor
        .get_all_client_statuses()
        .await
        .map_err(|e| {
            warn!("Error getting client statuses: {:#}", e);
            map_actor_error(e)
        })?;

    Ok(HttpResponse::Ok().json(clients))
}

/// Server-Sent Events endpoint for scan progress
#[get("/scan/progress")]
pub async fn scan_progress(_req: HttpRequest) -> Result<HttpResponse, Error> {
    debug!("New SSE client connected for scan progress");

    // Subscribe to progress updates
    let receiver = progress::subscribe();

    if receiver.is_none() {
        warn!("Progress broadcaster not initialized");
        return Err(Error::internal_server_error("Progress broadcaster not available"));
    }

    let mut rx = receiver.unwrap();

    // Create SSE stream
    let stream = async_stream::stream! {
        // Send initial connection message
        yield Ok::<_, actix_web::Error>(
            web::Bytes::from("event: connected\ndata: {}\n\n".to_string())
        );

        // Set up heartbeat interval
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(15));

        loop {
            tokio::select! {
                // Receive progress updates
                result = rx.recv() => {
                    match result {
                        Ok(progress) => {
                            // Serialize progress to JSON
                            match serde_json::to_string(&progress) {
                                Ok(json) => {
                                    yield Ok(web::Bytes::from(format!("data: {}\n\n", json)));
                                }
                                Err(e) => {
                                    error!("Failed to serialize progress: {}", e);
                                }
                            }
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                            warn!("SSE client lagged, skipped {} messages", skipped);
                            // Continue receiving
                            continue;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            debug!("Progress broadcaster closed");
                            break;
                        }
                    }
                }
                // Send periodic heartbeat to keep connection alive
                _ = heartbeat_interval.tick() => {
                    yield Ok(web::Bytes::from(": heartbeat\n\n"));
                }
            }
        }

        debug!("SSE client disconnected");
    };

    Ok(HttpResponse::Ok()
        .content_type("text/event-stream")
        .insert_header(("Cache-Control", "no-cache"))
        .insert_header(("X-Accel-Buffering", "no"))
        .streaming(Box::pin(stream)))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/monitoring")
            .service(get_status)
            .service(get_clients)
            .service(scan_progress)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}

pub fn configure_public(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/monitoring")
            .service(get_status)
            .service(get_clients)
            .service(scan_progress)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
