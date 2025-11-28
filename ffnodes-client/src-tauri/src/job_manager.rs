use crate::api::{EncodingJob, JobCompletion, ProgressUpdate, ServerClient, TransferProgress};
use crate::config::ClientConfig;
use crate::encoder::{Encoder, EncodingProgress};
use crate::gpu::GpuInfo;
use anyhow::{Result, anyhow};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct JobManagerState {
    pub is_processing: bool,
    pub is_paused: bool,
    pub current_job_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct JobStartedPayload {
    job: EncodingJob,
    total_frames: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TransferStartedPayload {
    job_id: String,
    filename: String,
    total_bytes: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct TransferCompletedPayload {
    job_id: String,
    total_bytes: u64,
    duration_secs: f64,
    avg_speed_mbps: f64,
}

pub struct JobManager {
    config: Arc<Mutex<Option<ClientConfig>>>,
    gpu_info: Arc<Mutex<Option<GpuInfo>>>,
    state: Arc<Mutex<JobManagerState>>,
    app_handle: AppHandle,
    encoder: Arc<Encoder>,
}

impl JobManager {
    pub async fn new(app_handle: AppHandle) -> Result<Self> {
        let encoder = Encoder::new().await?;

        Ok(Self {
            config: Arc::new(Mutex::new(None)),
            gpu_info: Arc::new(Mutex::new(None)),
            state: Arc::new(Mutex::new(JobManagerState {
                is_processing: false,
                is_paused: false,
                current_job_id: None,
            })),
            app_handle,
            encoder: Arc::new(encoder),
        })
    }

    pub async fn set_config(&self, config: ClientConfig) {
        let mut cfg = self.config.lock().await;
        *cfg = Some(config);
    }

    pub async fn set_gpu_info(&self, gpu: GpuInfo) {
        let mut gpu_info = self.gpu_info.lock().await;
        *gpu_info = Some(gpu);
    }

    pub async fn get_state(&self) -> JobManagerState {
        self.state.lock().await.clone()
    }

    pub async fn start(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        if state.is_processing {
            return Err(anyhow!("Job processing is already running"));
        }
        state.is_processing = true;
        state.is_paused = false;
        drop(state);

        log::info!("Starting job manager");

        // Start the processing loop in a background task
        let manager = Arc::new(self.clone_internals());
        tokio::spawn(async move {
            manager.processing_loop().await;
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        log::info!("Stopping job manager");

        // Get current job ID and client info before clearing state
        let (job_id, auth_token, server_url) = {
            let state = self.state.lock().await;
            let job_id = state.current_job_id.clone();

            // Get config for cancel request
            let config_lock = self.config.lock().await;
            let auth_token = config_lock.as_ref()
                .and_then(|c| c.auth_token.clone());
            let server_url = config_lock.as_ref()
                .map(|c| c.server_url.clone());

            (job_id, auth_token, server_url)
        };

        // If there's a current job, cancel it and clean up files
        if let Some(job_id) = job_id {
            log::info!("Cancelling current job {} and cleaning up files", job_id);

            // Cleanup temp files
            self.cleanup_job_files(&job_id).await;

            // Notify server to cancel the job
            if let (Some(auth_token), Some(server_url)) = (auth_token, server_url) {
                let client = ServerClient::with_auth(server_url, auth_token);
                match client.cancel_job(&job_id).await {
                    Ok(_) => log::info!("✓ Job {} cancelled on server", job_id),
                    Err(e) => log::warn!("Failed to cancel job {} on server: {}", job_id, e),
                }
            }
        }

        // Update state
        let mut state = self.state.lock().await;
        state.is_processing = false;
        state.is_paused = false;
        state.current_job_id = None;

        Ok(())
    }

    async fn cleanup_job_files(&self, job_id: &str) {
        log::debug!("Cleaning up temporary files for job {}", job_id);

        let temp_dir = std::env::temp_dir().join("ffnodes-client");

        // Delete input file
        let input_path = temp_dir.join(format!("input_{}", job_id));
        if input_path.exists() {
            match tokio::fs::remove_file(&input_path).await {
                Ok(_) => log::info!("✓ Deleted input file: {:?}", input_path),
                Err(e) => log::warn!("Failed to delete input file: {}", e),
            }
        }

        // Delete output files (glob pattern for any extension)
        // Try common output extensions
        for ext in &["mp4", "mkv", "webm", "avi", "mov"] {
            let output_path = temp_dir.join(format!("output_{}.{}", job_id, ext));
            if output_path.exists() {
                match tokio::fs::remove_file(&output_path).await {
                    Ok(_) => log::info!("✓ Deleted output file: {:?}", output_path),
                    Err(e) => log::warn!("Failed to delete output file: {}", e),
                }
            }
        }

        log::debug!("Cleanup complete for job {}", job_id);
    }

    pub async fn pause(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.is_paused = true;
        log::info!("Pausing job processing");
        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        let mut state = self.state.lock().await;
        state.is_paused = false;
        log::info!("Resuming job processing");
        Ok(())
    }

    fn clone_internals(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            gpu_info: Arc::clone(&self.gpu_info),
            state: Arc::clone(&self.state),
            app_handle: self.app_handle.clone(),
            encoder: Arc::clone(&self.encoder),
        }
    }

    async fn processing_loop(&self) {
        log::info!("Job processing loop started");

        loop {
            // Check if we should continue
            let state = self.state.lock().await;
            if !state.is_processing {
                log::info!("Job processing stopped");
                break;
            }
            let is_paused = state.is_paused;
            drop(state);

            if is_paused {
                log::debug!("Job processing is paused");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }

            // Try to process a job
            match self.process_next_job().await {
                Ok(processed) => {
                    if !processed {
                        // No jobs available, wait before polling again
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    }
                }
                Err(e) => {
                    log::error!("Error processing job: {}", e);
                    let _ = self.app_handle.emit("job-error", format!("Error: {}", e));
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn process_next_job(&self) -> Result<bool> {
        log::debug!("========== Starting process_next_job ==========");

        // Get config
        log::debug!("Acquiring config lock...");
        let config_lock = self.config.lock().await;
        let Some(config) = config_lock.as_ref() else {
            log::error!("No configuration set - cannot process jobs");
            return Err(anyhow!("No configuration set"));
        };
        let config = config.clone();
        drop(config_lock);
        log::debug!("Config acquired - server URL: {}", config.server_url);

        // Get GPU info
        log::debug!("Acquiring GPU info lock...");
        let gpu_lock = self.gpu_info.lock().await;
        let Some(gpu_info) = gpu_lock.as_ref() else {
            log::error!("No GPU information set");
            return Err(anyhow!("No GPU information set"));
        };
        let gpu_info = gpu_info.clone();
        drop(gpu_lock);
        log::debug!(
            "GPU info acquired - vendor: {}, name: {}, h264: {}, h265: {}",
            gpu_info.vendor,
            gpu_info.name,
            gpu_info.encoder_h264,
            gpu_info.encoder_h265
        );

        // Create API client with auth token
        log::debug!("Creating authenticated API client...");
        let auth_token = config
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow!("No auth token"))?
            .clone();
        let client = ServerClient::with_auth(config.server_url.clone(), auth_token.clone());

        // Request a job
        let client_id = config
            .client_id
            .as_ref()
            .ok_or_else(|| anyhow!("No client ID"))?;

        log::info!("Requesting job from server (client ID: {})", client_id);
        log::debug!(
            "API endpoint: {}/api/jobs/request/{}",
            config.server_url,
            client_id
        );

        let job_response = match client.request_job(client_id).await {
            Ok(resp) => {
                log::debug!("Job request successful");
                resp
            }
            Err(e) => {
                log::error!("Failed to request job from server: {:#}", e);
                return Err(e);
            }
        };

        let Some(job_resp) = job_response else {
            log::info!("No jobs available from server (received 204 No Content)");
            return Ok(false);
        };

        let job = job_resp.job;
        let job_id = job.id.clone();

        log::info!("✓ Received job from server: {}", job_id);
        log::debug!(
            "Job details - media_file_path: {}, status: {}",
            job.media_file_path,
            job.status
        );
        log::debug!("Output template: {}", job_resp.output_template);

        // Update state
        log::debug!("Updating job manager state...");
        {
            let mut state = self.state.lock().await;
            state.current_job_id = Some(job_id.clone());
        }
        log::debug!("State updated - current_job_id set to {}", job_id);

        // Emit job started event
        log::debug!("Emitting 'job-started' event to frontend...");
        let payload = JobStartedPayload {
            job: job.clone(),
            total_frames: job_resp.total_frames,
        };
        match self.app_handle.emit("job-started", &payload) {
            Ok(_) => log::debug!("✓ 'job-started' event emitted successfully"),
            Err(e) => log::warn!("Failed to emit 'job-started' event: {}", e),
        }

        // Start the job on server
        log::info!(
            "Starting job {} on server (status transition: assigned → in_progress)",
            job_id
        );
        log::debug!("Calling POST /api/jobs/{}/start", job_id);
        match client.start_job(&job_id).await {
            Ok(_) => log::info!("✓ Job {} started successfully on server", job_id),
            Err(e) => {
                log::error!("✗ Failed to start job {} on server: {:#}", job_id, e);
                return Err(e);
            }
        }

        // Process the job with error handling
        let result: Result<(i64, i64, f64)> = async {
            // Download input file
            log::info!("---------- Download Phase ----------");
            log::debug!("Creating temp directory...");
            let temp_dir = std::env::temp_dir().join("ffnodes-client");
            std::fs::create_dir_all(&temp_dir)?;
            log::debug!("✓ Temp directory created/verified: {:?}", temp_dir);

            let input_path = temp_dir.join(format!("input_{}", job_id));
            log::info!("Downloading input file for job {}", job_id);
            log::debug!("Download destination: {:?}", input_path);
            log::debug!("Calling GET /api/files/{}/input", job_id);

            // Prepare for download progress tracking
            let filename = job_resp
                .input_path
                .split('/')
                .last()
                .or_else(|| job_resp.input_path.split('\\').last())
                .unwrap_or("input_file")
                .to_string();

            // Track download start time
            let download_start = std::time::Instant::now();

            // Create progress callback for download
            // Use a simple bool that gets captured by the mutable closure
            let app_handle_clone = self.app_handle.clone();
            let job_id_clone = job_id.clone();
            let filename_clone = filename.clone();
            let mut first_progress = true;

            let mut progress_callback = move |progress: TransferProgress| {
                // Emit download-started on first progress update
                if first_progress {
                    let _ = app_handle_clone.emit("download-started", TransferStartedPayload {
                        job_id: job_id_clone.clone(),
                        filename: filename_clone.clone(),
                        total_bytes: progress.total_bytes,
                    });
                    first_progress = false;
                }

                // Emit download progress
                let _ = app_handle_clone.emit("download-progress", &progress);
            };

            // Download with progress tracking
            client.download_input_file(&job_id, &input_path, progress_callback).await?;

            // Emit download-completed event
            let download_duration = download_start.elapsed().as_secs_f64();
            if let Ok(metadata) = std::fs::metadata(&input_path) {
                let total_bytes = metadata.len();
                let avg_speed_mbps = (total_bytes as f64 / download_duration) / 1_000_000.0;

                log::info!("✓ Downloaded input file to {:?}", input_path);
                log::debug!("File size: {} bytes", total_bytes);
                log::debug!("Download duration: {:.2}s, avg speed: {:.2} MB/s", download_duration, avg_speed_mbps);

                let _ = self.app_handle.emit("download-completed", TransferCompletedPayload {
                    job_id: job_id.clone(),
                    total_bytes,
                    duration_secs: download_duration,
                    avg_speed_mbps,
                });
            }

            // Extract frame for background
            log::info!("---------- Frame Extraction Phase ----------");
            log::debug!("Extracting random frame for UI background...");
            let frame_base64 = self.encoder.extract_frame(&input_path).await?;
            log::info!(
                "✓ Frame extracted successfully (base64 length: {} chars)",
                frame_base64.len()
            );

            // Emit frame extracted event
            log::debug!("Emitting 'frame-extracted' event to frontend...");
            match self.app_handle.emit("frame-extracted", &frame_base64) {
                Ok(_) => log::debug!("✓ 'frame-extracted' event emitted successfully"),
                Err(e) => log::warn!("Failed to emit 'frame-extracted' event: {}", e),
            }

            // Prepare output path using server-provided container format
            let output_path = temp_dir.join(format!("output_{}.{}", job_id, job_resp.output_container));
            log::debug!("Output file will be saved to: {:?}", output_path);
            log::debug!("Output container: {}", job_resp.output_container);

            // Start encoding
            log::info!("---------- Encoding Phase ----------");
            log::info!("Starting encoding for job {}", job_id);
            log::debug!("Input: {:?}", input_path);
            log::debug!("Output: {:?}", output_path);
            log::debug!(
                "GPU: vendor={}, name={}, h264={}, h265={}",
                gpu_info.vendor,
                gpu_info.name,
                gpu_info.encoder_h264,
                gpu_info.encoder_h265
            );

            // Get FFmpeg command template from job response (server provides it)
            let ffmpeg_template = &job_resp.ffmpeg_template;
            log::debug!("FFmpeg template from server: {}", ffmpeg_template);
            log::debug!("Output template: {}", job_resp.output_template);

            let job_id_clone = job_id.clone();
            let app_handle = self.app_handle.clone();
            let client_clone = client.clone();

            let (output_size, output_bitrate, average_speed) = self
                .encoder
                .encode_video(
                    &input_path,
                    &output_path,
                    &gpu_info,
                    ffmpeg_template,
                    job_resp.total_frames,
                    move |progress: EncodingProgress| {
                        // Emit progress event
                        let _ = app_handle.emit("encoding-progress", &progress);

                        // Send progress update to server (throttled)
                        let client = client_clone.clone();
                        let job_id = job_id_clone.clone();
                        tokio::spawn(async move {
                            let update = ProgressUpdate {
                                frame: progress.frame,
                                fps: progress.fps,
                                bitrate: progress.bitrate,
                                speed: progress.speed,
                            };
                            let _ = client.update_progress(&job_id, update).await;
                        });
                    },
                )
                .await?;

            log::info!("✓ Encoding completed successfully for job {}", job_id);
            log::debug!(
                "Output file size: {} bytes, bitrate: {} bps, average speed: {:.2}x",
                output_size,
                output_bitrate,
                average_speed
            );

            // Upload output file
            log::info!("---------- Upload Phase ----------");
            log::info!("Uploading output file for job {}", job_id);
            log::debug!("Upload source: {:?}", output_path);
            log::debug!("Calling POST /api/files/{}/output", job_id);

            // Get file metadata for upload progress
            let upload_metadata = tokio::fs::metadata(&output_path).await?;
            let upload_total_bytes = upload_metadata.len();
            let upload_filename = output_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("output.mp4")
                .to_string();

            // Emit upload-started event
            self.app_handle.emit("upload-started", TransferStartedPayload {
                job_id: job_id.clone(),
                filename: upload_filename.clone(),
                total_bytes: upload_total_bytes,
            })?;

            // Track upload start time
            let upload_start = std::time::Instant::now();

            // Create progress callback for upload
            let app_handle_clone = self.app_handle.clone();

            let upload_progress_callback = move |progress: TransferProgress| {
                // Emit upload progress
                let _ = app_handle_clone.emit("upload-progress", &progress);
            };

            // Upload with progress tracking
            client.upload_output_file(&job_id, &output_path, upload_progress_callback).await?;

            // Emit upload-completed event
            let upload_duration = upload_start.elapsed().as_secs_f64();
            let avg_upload_speed_mbps = (upload_total_bytes as f64 / upload_duration) / 1_000_000.0;

            log::info!("✓ Upload completed successfully for job {}", job_id);
            log::debug!("Upload duration: {:.2}s, avg speed: {:.2} MB/s", upload_duration, avg_upload_speed_mbps);

            self.app_handle.emit("upload-completed", TransferCompletedPayload {
                job_id: job_id.clone(),
                total_bytes: upload_total_bytes,
                duration_secs: upload_duration,
                avg_speed_mbps: avg_upload_speed_mbps,
            })?;

            // Clean up temp files
            log::debug!("Cleaning up temporary files...");
            match tokio::fs::remove_file(&input_path).await {
                Ok(_) => log::debug!("✓ Deleted input file: {:?}", input_path),
                Err(e) => log::warn!("Failed to delete input file: {}", e),
            }
            match tokio::fs::remove_file(&output_path).await {
                Ok(_) => log::debug!("✓ Deleted output file: {:?}", output_path),
                Err(e) => log::warn!("Failed to delete output file: {}", e),
            }

            log::debug!("Returning success with metrics");
            Ok((output_size, output_bitrate, average_speed))
        }
        .await;

        match result {
            Ok((output_size, output_bitrate, average_speed)) => {
                log::info!("---------- Completion Phase ----------");
                log::debug!("Job processing succeeded, marking as complete on server...");

                // Complete the job
                let completion = JobCompletion {
                    output_size,
                    output_bitrate,
                    average_speed,
                };
                log::debug!("Calling POST /api/jobs/{}/complete with metrics", job_id);
                client.complete_job(&job_id, completion).await?;

                log::info!("✓✓✓ Job {} completed successfully ✓✓✓", job_id);

                // Update state
                log::debug!("Updating job manager state (clearing current_job_id)...");
                {
                    let mut state = self.state.lock().await;
                    state.current_job_id = None;
                }
                log::debug!("✓ State updated");

                // Emit job completed event
                log::debug!("Emitting 'job-completed' event to frontend...");
                match self.app_handle.emit("job-completed", &job_id) {
                    Ok(_) => log::debug!("✓ 'job-completed' event emitted successfully"),
                    Err(e) => log::warn!("Failed to emit 'job-completed' event: {}", e),
                }

                log::info!("========== process_next_job completed successfully ==========\n");
                Ok(true)
            }
            Err(e) => {
                log::error!("---------- Error Handling Phase ----------");
                log::error!("✗✗✗ Job {} failed: {:#} ✗✗✗", job_id, e);

                // Clean up temp files
                log::debug!("Cleaning up temporary files after job failure...");
                self.cleanup_job_files(&job_id).await;

                // Fail the job on server
                let error_message = format!("{:#}", e);
                log::debug!(
                    "Calling POST /api/jobs/{}/fail with error: {}",
                    job_id,
                    error_message
                );

                if let Err(fail_err) = client.fail_job(&job_id, error_message).await {
                    log::error!("✗ Failed to report job failure to server: {:#}", fail_err);
                } else {
                    log::info!("✓ Job failure reported to server successfully");
                }

                // Update state
                log::debug!("Updating job manager state (clearing current_job_id)...");
                {
                    let mut state = self.state.lock().await;
                    state.current_job_id = None;
                }
                log::debug!("✓ State updated");

                // Emit job error event
                log::debug!("Emitting 'job-error' event to frontend...");
                let error_msg = format!("Job failed: {:#}", e);
                match self.app_handle.emit("job-error", &error_msg) {
                    Ok(_) => log::debug!("✓ 'job-error' event emitted successfully"),
                    Err(emit_err) => log::warn!("Failed to emit 'job-error' event: {}", emit_err),
                }

                log::error!("========== process_next_job failed ==========\n");
                Err(e)
            }
        }
    }
}
