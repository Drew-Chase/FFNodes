use crate::clients::ClientManager;
use crate::http_error::Error;
use crate::jobs::JobQueue;
use actix_web::{web, HttpResponse};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
