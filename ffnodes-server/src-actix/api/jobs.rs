use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::job_actor::{ActorError, JobActorHandle};
use crate::jobs::{JobCompletion, JobFailure, JobResponse, ProgressUpdate};
use actix_web::{get, post, web, HttpResponse};
use tracing::{debug, warn, error, info, instrument};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

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

/// Request next job
#[post("/request/{client_id}")]
#[instrument(name = "request_job", skip(actor, config), fields(client_id = %client_id.as_ref()))]
pub async fn request_job(
    client_id: web::Path<String>,
    actor: web::Data<JobActorHandle>,
    config: web::Data<Arc<Configuration>>,
) -> Result<HttpResponse, Error> {
    info!("Processing job request from client");

    // Verify client is connected
    let is_connected = actor
        .is_client_connected(client_id.to_string())
        .await
        .map_err(|e| {
            error!("Error checking client connection: {:#}", e);
            map_actor_error(e)
        })?;

    if !is_connected {
        warn!("Job request rejected - client not connected: {}", client_id);
        return Err(Error::unauthorized_with_code("Client not connected", "CLIENT_NOT_CONNECTED"));
    }

    // Use atomic claim operation (replaces get_next_job + assign_job)
    let job = actor
        .claim_next_job(client_id.to_string())
        .await
        .map_err(|e| {
            error!("Error claiming job: {:#}", e);
            map_actor_error(e)
        })?;

    match job {
        Some(job) => {
            info!("Job claimed: job_id={}, path={}", job.id, job.media_file_path);

            // Get frame count from media_files table
            let total_frames = actor
                .get_media_file_frames(job.media_file_path.clone())
                .await
                .map_err(|e| {
                    error!("Error getting media file frames: {:#}", e);
                    map_actor_error(e)
                })?;

            let response = JobResponse {
                input_path: job.media_file_path.clone(),
                output_template: format!("{}.h264.{}", job.media_file_path, config.output_container),
                total_frames,
                ffmpeg_template: config.ffmpeg_template.clone(),
                output_container: config.output_container.clone(),
                job,
            };

            info!("Job claimed successfully");
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
#[instrument(name = "update_progress", skip(actor, ws_registry, progress), fields(job_id = %job_id.as_ref(), frame = progress.frame))]
pub async fn update_progress(
    job_id: web::Path<String>,
    progress: web::Json<ProgressUpdate>,
    actor: web::Data<JobActorHandle>,
    ws_registry: web::Data<crate::api::websocket::WsRegistry>,
) -> Result<HttpResponse, Error> {
    debug!("Progress update for job: frame={}, fps={}", progress.frame, progress.fps);

    let progress_data = progress.into_inner();

    // Update progress in database
    actor
        .update_progress(job_id.to_string(), progress_data.clone())
        .await
        .map_err(|e| {
            error!("Error updating progress: {:#}", e);
            map_actor_error(e)
        })?;

    // Get job details for broadcasting
    if let Ok(Some(job)) = actor.get_job(job_id.to_string()).await {
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
#[instrument(name = "update_phase", skip(actor), fields(job_id = %job_id.as_ref(), phase = %phase.phase))]
pub async fn update_phase(
    job_id: web::Path<String>,
    phase: web::Json<PhaseUpdate>,
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    info!("Phase update for job: phase={}", phase.phase);

    actor
        .update_phase(job_id.to_string(), phase.phase.clone())
        .await
        .map_err(|e| {
            error!("Error updating phase: {:#}", e);
            map_actor_error(e)
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Mark job as in progress
#[post("/{job_id}/start")]
#[instrument(name = "start_job", skip(actor), fields(job_id = %job_id.as_ref()))]
pub async fn start_job(
    job_id: web::Path<String>,
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    info!("Marking job as started");

    actor
        .start_job(job_id.to_string())
        .await
        .map_err(|e| {
            error!("Error starting job: {:#}", e);
            map_actor_error(e)
        })?;

    info!("Job started successfully");
    Ok(HttpResponse::Ok().finish())
}

/// Complete job
#[post("/{job_id}/complete")]
#[instrument(name = "complete_job", skip(actor, config, completion), fields(job_id = %job_id.as_ref(), output_size = completion.output_size))]
pub async fn complete_job(
    job_id: web::Path<String>,
    completion: web::Json<JobCompletion>,
    actor: web::Data<JobActorHandle>,
    config: web::Data<Arc<Configuration>>,
) -> Result<HttpResponse, Error> {
    info!("Completing job: output_size={}, bitrate={}, avg_speed={}",
        completion.output_size, completion.output_bitrate, completion.average_speed);

    // Get job to extract output path
    let job = actor
        .get_job(job_id.to_string())
        .await
        .map_err(|e| {
            error!("Error getting job: {:#}", e);
            map_actor_error(e)
        })?;

    let job = job.ok_or_else(|| {
        warn!("Job not found: {}", job_id);
        Error::not_found_with_code("Job not found", "JOB_NOT_FOUND")
    })?;

    let output_path = format!("{}.h264.{}", job.media_file_path, config.output_container);

    actor
        .complete_job(
            job_id.to_string(),
            output_path.clone(),
            completion.output_size,
            completion.output_bitrate,
            completion.average_speed,
        )
        .await
        .map_err(|e| {
            error!("Error completing job: {:#}", e);
            map_actor_error(e)
        })?;

    info!("Job completed successfully: output_path={}", output_path);
    Ok(HttpResponse::Ok().finish())
}

/// Fail job
#[post("/{job_id}/fail")]
#[instrument(name = "fail_job", skip(actor, failure), fields(job_id = %job_id.as_ref(), error = %failure.error))]
pub async fn fail_job(
    job_id: web::Path<String>,
    failure: web::Json<JobFailure>,
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    warn!("Marking job as failed: error={}", failure.error);

    actor
        .fail_job(job_id.to_string(), failure.error.clone())
        .await
        .map_err(|e| {
            error!("Error failing job: {:#}", e);
            map_actor_error(e)
        })?;

    info!("Job marked as failed");
    Ok(HttpResponse::Ok().finish())
}

/// Cancel job and requeue it
#[post("/{job_id}/cancel")]
#[instrument(name = "cancel_job", skip(actor), fields(job_id = %job_id.as_ref()))]
pub async fn cancel_job(
    job_id: web::Path<String>,
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    info!("Cancelling and requeuing job");

    actor
        .requeue_job(job_id.to_string())
        .await
        .map_err(|e| {
            error!("Error requeuing job: {:#}", e);
            map_actor_error(e)
        })?;

    info!("Job cancelled and requeued successfully");
    Ok(HttpResponse::Ok().finish())
}

/// Heartbeat endpoint
#[post("/heartbeat/{client_id}")]
#[instrument(name = "heartbeat", skip(actor), fields(client_id = %client_id.as_ref()))]
pub async fn heartbeat(
    client_id: web::Path<String>,
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    debug!("Heartbeat received");

    actor
        .update_heartbeat(client_id.to_string())
        .await
        .map_err(|e| {
            warn!("Error updating heartbeat: {:#}", e);
            Error::unauthorized_with_code("Client not found or disconnected", "CLIENT_NOT_CONNECTED")
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Client disconnect endpoint
#[post("/clients/{client_id}/disconnect")]
#[instrument(name = "disconnect_client", skip(actor), fields(client_id = %client_id.as_ref()))]
pub async fn disconnect_client(
    client_id: web::Path<String>,
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    info!("Processing client disconnect request");

    actor
        .disconnect_and_requeue_jobs(client_id.to_string())
        .await
        .map_err(|e| {
            error!("Error disconnecting client: {:#}", e);
            map_actor_error(e)
        })?;

    info!("Client disconnected successfully");
    Ok(HttpResponse::Ok().finish())
}

/// Get active jobs
#[get("/active")]
#[instrument(name = "get_active_jobs", skip(actor))]
pub async fn get_active_jobs(
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    debug!("Retrieving active jobs");

    let jobs = actor
        .get_active_jobs()
        .await
        .map_err(|e| {
            error!("Error getting active jobs: {:#}", e);
            map_actor_error(e)
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
