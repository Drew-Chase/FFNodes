use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::jobs::JobQueue;
use crate::path_security;
use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse};
use futures_util::StreamExt;
use log::{debug, warn};
use serde_json::json;
use std::fs::File;
use std::io::Write as _;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;

/// Download input file for a job
/// GET /api/files/{job_id}/input
#[get("/{job_id}/input")]
pub async fn download_input(
    job_id: web::Path<String>,
    job_queue: web::Data<Arc<JobQueue>>,
    config: web::Data<Arc<Configuration>>,
) -> Result<HttpResponse, Error> {
    debug!("Download request for job: {}", job_id);

    // Get job
    let job = job_queue.get_job(&job_id).await.map_err(|e| {
        warn!("Error getting job: {:#}", e);
        Error::internal_server_error("Error getting job")
    })?;

    let job = job.ok_or_else(|| {
        warn!("Job not found: {}", job_id);
        Error::not_found("Job not found")
    })?;

    debug!("Job found - media_file_path: {}", job.media_file_path);

    // Verify job is assigned
    if job.assigned_client.is_none() {
        warn!("Job not assigned to any client");
        return Err(Error::bad_request("Job not assigned"));
    }

    debug!("Job assigned to client: {:?}", job.assigned_client);

    let input_path = Path::new(&job.media_file_path);
    debug!("Input path: {:?}", input_path);
    debug!("Watch directories: {:?}", config.watch_directories);

    // Validate path security - ensure no path traversal
    let validated_path = path_security::validate_path_within_base(
        input_path,
        &config.watch_directories,
    )
    .map_err(|e| {
        warn!("Path validation failed: {:#}", e);
        Error::forbidden("Access to this path is not allowed")
    })?;

    debug!("Path validated: {:?}", validated_path);

    // Check if file exists
    if !validated_path.exists() {
        return Err(Error::not_found("Input file not found"));
    }

    // Get file metadata
    let metadata = tokio::fs::metadata(&validated_path).await.map_err(|e| {
        warn!("Error reading file metadata: {:#}", e);
        Error::internal_server_error("Error reading file metadata")
    })?;

    // Read file
    let mut file = TokioFile::open(&validated_path).await.map_err(|e| {
        warn!("Error opening file: {:#}", e);
        Error::internal_server_error("Error opening file")
    })?;

    let mut buffer = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer).await.map_err(|e| {
        warn!("Error reading file: {:#}", e);
        Error::internal_server_error("Error reading file")
    })?;

    // Get filename for Content-Disposition
    let filename = validated_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("video.mp4");

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .insert_header((
            "Content-Disposition",
            format!("attachment; filename=\"{}\"", filename),
        ))
        .body(buffer))
}

/// Upload output file for a job
/// POST /api/files/{job_id}/output
#[post("/{job_id}/output")]
pub async fn upload_output(
    job_id: web::Path<String>,
    mut payload: Multipart,
    job_queue: web::Data<Arc<JobQueue>>,
    config: web::Data<Arc<Configuration>>,
) -> Result<HttpResponse, Error> {
    debug!("Upload request for job: {}", job_id);

    // Get job
    let job = job_queue.get_job(&job_id).await.map_err(|e| {
        warn!("Error getting job: {:#}", e);
        Error::internal_server_error("Error getting job")
    })?;

    let job = job.ok_or_else(|| {
        warn!("Job not found: {}", job_id);
        Error::not_found("Job not found")
    })?;

    debug!("Job found - status: {}, media_file_path: {}", job.status, job.media_file_path);

    // Verify job is in progress
    if job.status != "in_progress" {
        warn!("Upload rejected - job status is '{}', expected 'in_progress'", job.status);
        return Err(Error::bad_request(format!("Job is not in progress (status: {})", job.status)));
    }

    debug!("✓ Job status verified: in_progress");

    // Calculate output path by replacing extension with output_container
    let original_path = Path::new(&job.media_file_path);
    let output_path = if let Some(parent) = original_path.parent() {
        let stem = original_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::internal_server_error("Invalid filename"))?;
        parent.join(format!("{}.{}", stem, config.output_container))
    } else {
        // No parent directory, just use filename
        let stem = original_path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::internal_server_error("Invalid filename"))?;
        Path::new(&format!("{}.{}", stem, config.output_container)).to_path_buf()
    };

    debug!("Original file path: {}", original_path.display());
    debug!("Calculated output path: {}", output_path.display());

    // Validate path security - ensure no path traversal
    debug!("Validating path security...");
    let validated_path = path_security::validate_path_security_non_existent(&output_path).map_err(|e| {
        warn!("Path validation failed: {:#}", e);
        Error::forbidden("Invalid output path")
    })?;

    debug!("✓ Path security validated: {:?}", validated_path);

    // Process multipart stream
    debug!("Processing multipart upload stream...");
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            warn!("Error reading multipart field: {:#}", e);
            Error::bad_request("Invalid multipart data")
        })?;

        debug!("Reading multipart field...");
        // Read field data
        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| {
                warn!("Error reading chunk: {:#}", e);
                Error::bad_request("Error reading file data")
            })?;
            data.extend_from_slice(&chunk);
        }

        debug!("Field data size: {} bytes", data.len());
        file_data = Some(data);
    }

    let file_data = file_data.ok_or_else(|| {
        warn!("No file data received in multipart upload");
        Error::bad_request("No file provided")
    })?;

    debug!("✓ Multipart upload processed - total size: {} bytes", file_data.len());

    // Write file to output path
    let mut file = File::create(&validated_path).map_err(|e| {
        warn!("Error creating output file: {:#}", e);
        Error::internal_server_error("Error creating output file")
    })?;

    file.write_all(&file_data).map_err(|e| {
        warn!("Error writing output file: {:#}", e);
        Error::internal_server_error("Error writing output file")
    })?;

    debug!(
        "Successfully uploaded {} bytes to {}",
        file_data.len(),
        validated_path.display()
    );

    // Delete the original file if it's different from the output path
    if original_path != validated_path && original_path.exists() {
        debug!("Deleting original file: {}", original_path.display());
        std::fs::remove_file(original_path).map_err(|e| {
            warn!("Error deleting original file: {:#}", e);
            // Don't fail the upload if deletion fails, just warn
            // The encoded file has already been written successfully
            warn!("Failed to delete original file, but upload was successful");
        }).ok(); // Ignore deletion errors
        debug!("✓ Original file deleted");
    } else if original_path == validated_path {
        debug!("Output path is same as original, skipping deletion (overwrite)");
    } else {
        debug!("Original file doesn't exist, skipping deletion");
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "File uploaded successfully",
        "size": file_data.len(),
        "path": validated_path.display().to_string(),
        "original_deleted": original_path != validated_path && !original_path.exists()
    })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/files")
            .service(download_input)
            .service(upload_output)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
