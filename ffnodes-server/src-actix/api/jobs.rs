use crate::clients::ClientManager;
use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::jobs::{JobCompletion, JobFailure, JobQueue, JobResponse, ProgressUpdate};
use actix_web::{get, post, web, HttpResponse};
use anyhow::Context;
use tracing::{debug, warn, error, info, instrument};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

/// Request next job
#[post("/request/{client_id}")]
#[instrument(name = "request_job", skip(job_queue, client_manager, config), fields(client_id = %client_id.as_ref()))]
pub async fn request_job(
    client_id: web::Path<String>,
    job_queue: web::Data<Arc<JobQueue>>,
    client_manager: web::Data<Arc<ClientManager>>,
    config: web::Data<Arc<Configuration>>,
) -> Result<HttpResponse, Error> {
    info!("Processing job request from client");

    // Verify client is connected
    if !client_manager
        .is_client_connected(&client_id)
        .await
        .context(format!("Failed to check client connection status: client_id={}", client_id))
        .map_err(|e| {
            error!("Error checking client connection: {:#}", e);
            Error::internal_server_error("Error checking client connection")
        })?
    {
        warn!("Job request rejected - client not connected: {}", client_id);
        return Err(Error::unauthorized_with_code("Client not connected", "CLIENT_NOT_CONNECTED"));
    }

    // Get next job
    let job = job_queue
        .get_next_job()
        .await
        .context("Failed to retrieve next pending job from queue")
        .map_err(|e| {
            error!("Error getting next job: {:#}", e);
            Error::internal_server_error("Error getting next job")
        })?;

    match job {
        Some(mut job) => {
            info!("Found pending job: job_id={}, path={}", job.id, job.media_file_path);

            // Assign job to client
            job_queue
                .assign_job(&job.id, &client_id)
                .await
                .context(format!("Failed to assign job {} to client {}", job.id, client_id))
                .map_err(|e| {
                    error!("Error assigning job: {:#}", e);
                    Error::internal_server_error("Error assigning job")
                })?;

            job.assigned_client = Some(client_id.to_string());

            // Get frame count from media_files table
            let total_frames = job_queue
                .get_media_file_frames(&job.media_file_path)
                .await
                .context(format!("Failed to retrieve frame count for media file: path={}", job.media_file_path))
                .map_err(|e| {
                    error!("Error getting media file frames: {:#}", e);
                    Error::internal_server_error("Error getting media file frames")
                })?;

            let response = JobResponse {
                input_path: job.media_file_path.clone(),
                output_template: format!("{}.h264.{}", job.media_file_path, config.output_container),
                total_frames,
                ffmpeg_template: config.ffmpeg_template.clone(),
                output_container: config.output_container.clone(),
                job,
            };

            info!("Job assigned successfully");
            Ok(HttpResponse::Ok().json(response))
        }
        None => {
            debug!("No pending jobs available for client");
            Ok(HttpResponse::NoContent().finish())
        }
    }
}

/// Update job progress
#[post("/{job_id}/progress")]
#[instrument(name = "update_progress", skip(job_queue, ws_registry, progress), fields(job_id = %job_id.as_ref(), frame = progress.frame))]
pub async fn update_progress(
    job_id: web::Path<String>,
    progress: web::Json<ProgressUpdate>,
    job_queue: web::Data<Arc<JobQueue>>,
    ws_registry: web::Data<crate::api::websocket::WsRegistry>,
) -> Result<HttpResponse, Error> {
    debug!("Progress update for job: frame={}, fps={}", progress.frame, progress.fps);

    let progress_data = progress.into_inner();

    // Update progress in database
    job_queue
        .update_progress(&job_id, progress_data.clone())
        .await
        .context(format!("Failed to update job progress in database: job_id={}", job_id))
        .map_err(|e| {
            error!("Error updating progress: {:#}", e);
            Error::internal_server_error("Error updating progress")
        })?;

    // Get job details for broadcasting
    if let Ok(Some(job)) = job_queue.get_job(&job_id).await {
        if let Some(assigned_client) = job.assigned_client {
            // Broadcast progress event to all connected clients
            let event = crate::api::websocket::WsEvent::Progress {
                job_id: job_id.to_string(),
                client_id: assigned_client,
                frame: progress_data.frame,
                fps: progress_data.fps,
                speed: progress_data.speed.clone(),
            };

            crate::api::websocket::broadcast_event(&ws_registry, event).await;
        }
    }

    Ok(HttpResponse::Ok().finish())
}

/// Update job phase (downloading, encoding, uploading)
#[derive(Debug, Serialize, Deserialize)]
pub struct PhaseUpdate {
    pub phase: String,
}

#[post("/{job_id}/phase")]
#[instrument(name = "update_phase", skip(job_queue), fields(job_id = %job_id.as_ref(), phase = %phase.phase))]
pub async fn update_phase(
    job_id: web::Path<String>,
    phase: web::Json<PhaseUpdate>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    info!("Phase update for job: phase={}", phase.phase);

    job_queue
        .update_phase(&job_id, &phase.phase)
        .await
        .context(format!("Failed to update job phase: job_id={}, phase={}", job_id, phase.phase))
        .map_err(|e| {
            error!("Error updating phase: {:#}", e);
            Error::internal_server_error("Error updating phase")
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Mark job as in progress
#[post("/{job_id}/start")]
#[instrument(name = "start_job", skip(job_queue), fields(job_id = %job_id.as_ref()))]
pub async fn start_job(
    job_id: web::Path<String>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    info!("Marking job as started");

    job_queue
        .start_job(&job_id)
        .await
        .context(format!("Failed to mark job as in progress: job_id={}", job_id))
        .map_err(|e| {
            error!("Error starting job: {:#}", e);
            Error::internal_server_error("Error starting job")
        })?;

    info!("Job started successfully");
    Ok(HttpResponse::Ok().finish())
}

/// Complete job
#[post("/{job_id}/complete")]
#[instrument(name = "complete_job", skip(job_queue, config, completion), fields(job_id = %job_id.as_ref(), output_size = completion.output_size))]
pub async fn complete_job(
    job_id: web::Path<String>,
    completion: web::Json<JobCompletion>,
    job_queue: web::Data<Arc<JobQueue>>,
    config: web::Data<Arc<Configuration>>,
) -> Result<HttpResponse, Error> {
    info!("Completing job: output_size={}, bitrate={}, avg_speed={}",
        completion.output_size, completion.output_bitrate, completion.average_speed);

    // Get job to extract output path
    let job = job_queue
        .get_job(&job_id)
        .await
        .context(format!("Failed to retrieve job for completion: job_id={}", job_id))
        .map_err(|e| {
            error!("Error getting job: {:#}", e);
            Error::internal_server_error("Error getting job")
        })?;

    let job = job.ok_or_else(|| {
        warn!("Job not found: {}", job_id);
        Error::not_found_with_code("Job not found", "JOB_NOT_FOUND")
    })?;

    let output_path = format!("{}.h264.{}", job.media_file_path, config.output_container);

    job_queue
        .complete_job(
            &job_id,
            output_path.clone(),
            completion.output_size,
            completion.output_bitrate,
            completion.average_speed,
        )
        .await
        .context(format!("Failed to mark job as completed: job_id={}, output_path={}", job_id, output_path))
        .map_err(|e| {
            error!("Error completing job: {:#}", e);
            Error::internal_server_error("Error completing job")
        })?;

    info!("Job completed successfully: output_path={}", output_path);
    Ok(HttpResponse::Ok().finish())
}

/// Fail job
#[post("/{job_id}/fail")]
#[instrument(name = "fail_job", skip(job_queue, failure), fields(job_id = %job_id.as_ref(), error = %failure.error))]
pub async fn fail_job(
    job_id: web::Path<String>,
    failure: web::Json<JobFailure>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    warn!("Marking job as failed: error={}", failure.error);

    job_queue
        .fail_job(&job_id, failure.error.clone())
        .await
        .context(format!("Failed to mark job as failed: job_id={}, error={}", job_id, failure.error))
        .map_err(|e| {
            error!("Error failing job: {:#}", e);
            Error::internal_server_error("Error failing job")
        })?;

    info!("Job marked as failed");
    Ok(HttpResponse::Ok().finish())
}

/// Cancel job and requeue it
#[post("/{job_id}/cancel")]
#[instrument(name = "cancel_job", skip(job_queue), fields(job_id = %job_id.as_ref()))]
pub async fn cancel_job(
    job_id: web::Path<String>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    info!("Cancelling and requeuing job");

    job_queue
        .requeue_job(&job_id)
        .await
        .context(format!("Failed to requeue cancelled job: job_id={}", job_id))
        .map_err(|e| {
            error!("Error requeuing job: {:#}", e);
            Error::internal_server_error("Error requeuing job")
        })?;

    info!("Job cancelled and requeued successfully");
    Ok(HttpResponse::Ok().finish())
}

/// Heartbeat endpoint
#[post("/heartbeat/{client_id}")]
#[instrument(name = "heartbeat", skip(client_manager), fields(client_id = %client_id.as_ref()))]
pub async fn heartbeat(
    client_id: web::Path<String>,
    client_manager: web::Data<Arc<ClientManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Heartbeat received");

    client_manager
        .update_heartbeat(&client_id)
        .await
        .context(format!("Failed to update heartbeat timestamp: client_id={}", client_id))
        .map_err(|e| {
            warn!("Error updating heartbeat: {:#}", e);
            Error::unauthorized_with_code("Client not found or disconnected", "CLIENT_NOT_CONNECTED")
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Client disconnect endpoint
#[post("/clients/{client_id}/disconnect")]
#[instrument(name = "disconnect_client", skip(client_manager, job_queue), fields(client_id = %client_id.as_ref()))]
pub async fn disconnect_client(
    client_id: web::Path<String>,
    client_manager: web::Data<Arc<ClientManager>>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    info!("Processing client disconnect request");

    client_manager
        .disconnect_and_requeue_jobs(&client_id, &job_queue)
        .await
        .context(format!("Failed to disconnect client and requeue jobs: client_id={}", client_id))
        .map_err(|e| {
            error!("Error disconnecting client: {:#}", e);
            Error::internal_server_error("Error disconnecting client")
        })?;

    info!("Client disconnected successfully");
    Ok(HttpResponse::Ok().finish())
}

/// Get active jobs
#[get("/active")]
#[instrument(name = "get_active_jobs", skip(job_queue))]
pub async fn get_active_jobs(
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    debug!("Retrieving active jobs");

    let jobs = job_queue
        .get_active_jobs()
        .await
        .context("Failed to retrieve active jobs from database")
        .map_err(|e| {
            error!("Error getting active jobs: {:#}", e);
            Error::internal_server_error("Error getting active jobs")
        })?;

    info!("Retrieved {} active jobs", jobs.len());
    Ok(HttpResponse::Ok().json(jobs))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/jobs")
            .service(request_job)
            .service(start_job)
            .service(update_progress)
            .service(update_phase)
            .service(complete_job)
            .service(fail_job)
            .service(cancel_job)
            .service(heartbeat)
            .service(get_active_jobs)
    );
    cfg.service(
        web::scope("/clients")
            .service(disconnect_client)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}

pub fn configure_public(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/jobs")
            .service(get_active_jobs)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
