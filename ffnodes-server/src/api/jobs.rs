use crate::clients::ClientManager;
use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::jobs::{JobCompletion, JobFailure, JobQueue, JobResponse, ProgressUpdate};
use actix_web::{web, HttpResponse};
use log::{debug, warn};
use std::sync::Arc;

/// Request next job
pub async fn request_job(
    client_id: web::Path<String>,
    job_queue: web::Data<Arc<JobQueue>>,
    client_manager: web::Data<Arc<ClientManager>>,
    config: web::Data<Arc<Configuration>>,
) -> Result<HttpResponse, Error> {
    debug!("Job request from client: {}", client_id);

    // Verify client is connected
    if !client_manager
        .is_client_connected(&client_id)
        .await
        .map_err(|e| {
            warn!("Error checking client connection: {:#}", e);
            Error::internal_server_error("Error checking client connection")
        })?
    {
        return Err(Error::unauthorized("Client not connected"));
    }

    // Get next job
    let job = job_queue.get_next_job().await.map_err(|e| {
        warn!("Error getting next job: {:#}", e);
        Error::internal_server_error("Error getting next job")
    })?;

    match job {
        Some(mut job) => {
            // Assign job to client
            job_queue
                .assign_job(&job.id, &client_id)
                .await
                .map_err(|e| {
                    warn!("Error assigning job: {:#}", e);
                    Error::internal_server_error("Error assigning job")
                })?;

            job.assigned_client = Some(client_id.to_string());

            // Get frame count from media_files table
            let total_frames = job_queue
                .get_media_file_frames(&job.media_file_path)
                .await
                .map_err(|e| {
                    warn!("Error getting media file frames: {:#}", e);
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

            Ok(HttpResponse::Ok().json(response))
        }
        None => Ok(HttpResponse::NoContent().finish()),
    }
}

/// Update job progress
pub async fn update_progress(
    job_id: web::Path<String>,
    progress: web::Json<ProgressUpdate>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    debug!("Progress update for job: {}", job_id);

    // Update progress
    job_queue
        .update_progress(&job_id, progress.into_inner())
        .await
        .map_err(|e| {
            warn!("Error updating progress: {:#}", e);
            Error::internal_server_error("Error updating progress")
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Mark job as in progress
pub async fn start_job(
    job_id: web::Path<String>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    debug!("Starting job: {}", job_id);

    job_queue.start_job(&job_id).await.map_err(|e| {
        warn!("Error starting job: {:#}", e);
        Error::internal_server_error("Error starting job")
    })?;

    Ok(HttpResponse::Ok().finish())
}

/// Complete job
pub async fn complete_job(
    job_id: web::Path<String>,
    completion: web::Json<JobCompletion>,
    job_queue: web::Data<Arc<JobQueue>>,
    config: web::Data<Arc<Configuration>>,
) -> Result<HttpResponse, Error> {
    debug!("Completing job: {}", job_id);

    // Get job to extract output path
    let job = job_queue.get_job(&job_id).await.map_err(|e| {
        warn!("Error getting job: {:#}", e);
        Error::internal_server_error("Error getting job")
    })?;

    let job = job.ok_or_else(|| {
        warn!("Job not found: {}", job_id);
        Error::not_found("Job not found")
    })?;

    let output_path = format!("{}.h264.{}", job.media_file_path, config.output_container);

    job_queue
        .complete_job(
            &job_id,
            output_path,
            completion.output_size,
            completion.output_bitrate,
            completion.average_speed,
        )
        .await
        .map_err(|e| {
            warn!("Error completing job: {:#}", e);
            Error::internal_server_error("Error completing job")
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Fail job
pub async fn fail_job(
    job_id: web::Path<String>,
    failure: web::Json<JobFailure>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    debug!("Failing job: {}", job_id);

    job_queue
        .fail_job(&job_id, failure.error.clone())
        .await
        .map_err(|e| {
            warn!("Error failing job: {:#}", e);
            Error::internal_server_error("Error failing job")
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Cancel job and requeue it
pub async fn cancel_job(
    job_id: web::Path<String>,
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    debug!("Cancelling job: {}", job_id);

    job_queue
        .requeue_job(&job_id)
        .await
        .map_err(|e| {
            warn!("Error requeuing job: {:#}", e);
            Error::internal_server_error("Error requeuing job")
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Heartbeat endpoint
pub async fn heartbeat(
    client_id: web::Path<String>,
    client_manager: web::Data<Arc<ClientManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Heartbeat from client: {}", client_id);

    client_manager
        .update_heartbeat(&client_id)
        .await
        .map_err(|e| {
            warn!("Error updating heartbeat: {:#}", e);
            Error::unauthorized("Client not found or disconnected")
        })?;

    Ok(HttpResponse::Ok().finish())
}

/// Get active jobs
pub async fn get_active_jobs(
    job_queue: web::Data<Arc<JobQueue>>,
) -> Result<HttpResponse, Error> {
    debug!("Getting active jobs");

    let jobs = job_queue.get_active_jobs().await.map_err(|e| {
        warn!("Error getting active jobs: {:#}", e);
        Error::internal_server_error("Error getting active jobs")
    })?;

    Ok(HttpResponse::Ok().json(jobs))
}
