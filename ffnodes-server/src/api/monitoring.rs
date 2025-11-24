use crate::clients::ClientManager;
use crate::http_error::Error;
use crate::jobs::JobQueue;
use crate::media_files::progress;
use actix_web::{web, HttpResponse, HttpRequest};
use tracing::{debug, warn, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use futures::stream::StreamExt;

/// System status response
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStatus {
    pub total_media_files: i64,
    pub pending_jobs: i64,
    pub active_jobs: i64,
    pub connected_clients: usize,
}

/// Get overall system status
pub async fn get_status(
    job_queue: web::Data<Arc<JobQueue>>,
    client_manager: web::Data<Arc<ClientManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Getting system status");

    let pending_jobs = job_queue.get_pending_count().await.map_err(|e| {
        warn!("Error getting pending jobs count: {:#}", e);
        Error::internal_server_error("Error getting pending jobs count")
    })?;

    let active_jobs = job_queue.get_active_count().await.map_err(|e| {
        warn!("Error getting active jobs count: {:#}", e);
        Error::internal_server_error("Error getting active jobs count")
    })?;

    let clients = client_manager.get_connected_clients().await.map_err(|e| {
        warn!("Error getting connected clients: {:#}", e);
        Error::internal_server_error("Error getting connected clients")
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
pub async fn get_clients(
    client_manager: web::Data<Arc<ClientManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Getting all clients");

    let clients = client_manager
        .get_all_client_statuses()
        .await
        .map_err(|e| {
            warn!("Error getting client statuses: {:#}", e);
            Error::internal_server_error("Error getting client statuses")
        })?;

    Ok(HttpResponse::Ok().json(clients))
}

/// Server-Sent Events endpoint for scan progress
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
            web::Bytes::from(format!("event: connected\ndata: {{}}\n\n"))
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
