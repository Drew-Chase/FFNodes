use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::jobs::JobQueue;
use crate::path_security;
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures_util::StreamExt;
use log::{debug, warn};
use std::fs::File;
use std::io::Write as _;
use std::path::Path;
use std::sync::Arc;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncReadExt;

/// Download input file for a job
/// GET /api/files/{job_id}/input
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

    let output_path_str = format!("{}.h264.{}", job.media_file_path, config.output_container);
    debug!("Calculated output path: {}", output_path_str);
    let output_path = Path::new(&output_path_str);

    // Validate path security - ensure no path traversal
    // Use non-existent version since the output file doesn't exist yet
    debug!("Validating path security...");
    let validated_path = path_security::validate_path_security_non_existent(output_path).map_err(|e| {
        warn!("Path validation failed: {:#}", e);
        Error::forbidden("Invalid output path")
    })?;

    debug!("✓ Path security validated: {:?}", validated_path);

    // Ensure output directory is within watch directories
    debug!("Validating output directory within watch directories...");
    if let Some(parent) = validated_path.parent() {
        debug!("Parent directory: {:?}", parent);
        debug!("Watch directories: {:?}", config.watch_directories);

        // Use non-existent version in case parent directory doesn't exist yet
        path_security::validate_path_within_base_non_existent(parent, &config.watch_directories).map_err(
            |e| {
                warn!("Output directory validation failed: {:#}", e);
                Error::forbidden("Output directory is not allowed")
            },
        )?;
        debug!("✓ Output directory validated");

        // Create parent directory if it doesn't exist
        if !parent.exists() {
            debug!("Creating parent directory: {:?}", parent);
            std::fs::create_dir_all(parent).map_err(|e| {
                warn!("Failed to create parent directory: {:#}", e);
                Error::internal_server_error("Failed to create output directory")
            })?;
            debug!("✓ Parent directory created");
        }
    }

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

    // Write file
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

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "File uploaded successfully",
        "size": file_data.len(),
        "path": validated_path.display().to_string()
    })))
}
