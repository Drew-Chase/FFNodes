use crate::http_error::Error;
use crate::jobs::JobQueue;
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

    // Verify job is assigned
    if job.assigned_client.is_none() {
        return Err(Error::bad_request("Job not assigned"));
    }

    let input_path = Path::new(&job.media_file_path);

    // Check if file exists
    if !input_path.exists() {
        return Err(Error::not_found("Input file not found"));
    }

    // Get file metadata
    let metadata = tokio::fs::metadata(&input_path).await.map_err(|e| {
        warn!("Error reading file metadata: {:#}", e);
        Error::internal_server_error("Error reading file metadata")
    })?;

    // Read file
    let mut file = TokioFile::open(&input_path).await.map_err(|e| {
        warn!("Error opening file: {:#}", e);
        Error::internal_server_error("Error opening file")
    })?;

    let mut buffer = Vec::with_capacity(metadata.len() as usize);
    file.read_to_end(&mut buffer).await.map_err(|e| {
        warn!("Error reading file: {:#}", e);
        Error::internal_server_error("Error reading file")
    })?;

    // Get filename for Content-Disposition
    let filename = input_path
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

    // Verify job is in progress
    if job.status != "in_progress" {
        return Err(Error::bad_request("Job is not in progress"));
    }

    let output_path = format!("{}.h264.mp4", job.media_file_path);

    // Process multipart stream
    let mut file_data: Option<Vec<u8>> = None;

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| {
            warn!("Error reading multipart field: {:#}", e);
            Error::bad_request("Invalid multipart data")
        })?;

        // Read field data
        let mut data = Vec::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.map_err(|e| {
                warn!("Error reading chunk: {:#}", e);
                Error::bad_request("Error reading file data")
            })?;
            data.extend_from_slice(&chunk);
        }

        file_data = Some(data);
    }

    let file_data = file_data.ok_or_else(|| Error::bad_request("No file provided"))?;

    // Write file
    let mut file = File::create(&output_path).map_err(|e| {
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
        output_path
    );

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "File uploaded successfully",
        "size": file_data.len(),
        "path": output_path
    })))
}
