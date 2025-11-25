use crate::api::{JobCompletion, ProgressUpdate, ServerClient};
use crate::config::ClientConfig;
use crate::encoder::{Encoder, EncodingProgress};
use crate::gpu::GpuInfo;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct JobManagerState {
    pub is_processing: bool,
    pub is_paused: bool,
    pub current_job_id: Option<String>,
}

pub struct JobManager {
    config: Arc<Mutex<Option<ClientConfig>>>,
    gpu_info: Arc<Mutex<Option<GpuInfo>>>,
    state: Arc<Mutex<JobManagerState>>,
    app_handle: AppHandle,
    encoder: Arc<Encoder>,
}

impl JobManager {
    pub fn new(app_handle: AppHandle) -> Result<Self> {
        let encoder = Encoder::new()?;

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
        let mut state = self.state.lock().await;
        state.is_processing = false;
        state.is_paused = false;
        state.current_job_id = None;
        log::info!("Stopping job manager");
        Ok(())
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
                    let _ = self
                        .app_handle
                        .emit("job-error", format!("Error: {}", e));
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                }
            }
        }
    }

    async fn process_next_job(&self) -> Result<bool> {
        // Get config
        let config_lock = self.config.lock().await;
        let Some(config) = config_lock.as_ref() else {
            return Err(anyhow!("No configuration set"));
        };
        let config = config.clone();
        drop(config_lock);

        // Get GPU info
        let gpu_lock = self.gpu_info.lock().await;
        let Some(gpu_info) = gpu_lock.as_ref() else {
            return Err(anyhow!("No GPU information set"));
        };
        let gpu_info = gpu_info.clone();
        drop(gpu_lock);

        // Create API client
        let client = ServerClient::new(config.server_url.clone());

        // Request a job
        let client_id = config
            .auth_token
            .as_ref()
            .ok_or_else(|| anyhow!("No auth token"))?;

        log::debug!("Requesting job from server");
        let job_response = client.request_job(client_id).await?;

        let Some(job_resp) = job_response else {
            log::debug!("No jobs available");
            return Ok(false);
        };

        let job = job_resp.job;
        let job_id = job.id.clone();

        log::info!("Received job: {}", job_id);

        // Update state
        {
            let mut state = self.state.lock().await;
            state.current_job_id = Some(job_id.clone());
        }

        // Emit job started event
        let _ = self.app_handle.emit("job-started", &job);

        // Download input file
        log::info!("Downloading input file for job {}", job_id);
        let temp_dir = std::env::temp_dir().join("ffnodes-client");
        std::fs::create_dir_all(&temp_dir)?;

        let input_path = temp_dir.join(format!("input_{}", job_id));
        client
            .download_input_file(&job_id, &input_path)
            .await?;

        log::info!("Downloaded input file to {:?}", input_path);

        // Extract frame for background
        log::info!("Extracting frame for background");
        let frame_base64 = self.encoder.extract_frame(&input_path).await?;

        // Emit frame extracted event
        let _ = self.app_handle.emit("frame-extracted", &frame_base64);

        // Prepare output path
        let output_path = temp_dir.join(format!("output_{}", job_id));

        // Start encoding
        log::info!("Starting encoding for job {}", job_id);
        let job_id_clone = job_id.clone();
        let app_handle = self.app_handle.clone();
        let client_clone = client.clone();

        let (output_size, output_bitrate) = self
            .encoder
            .encode_video(
                &input_path,
                &output_path,
                &gpu_info,
                &job_resp.output_template,
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

        log::info!("Encoding completed for job {}", job_id);

        // Upload output file
        log::info!("Uploading output file for job {}", job_id);
        client.upload_output_file(&job_id, &output_path).await?;

        log::info!("Upload completed for job {}", job_id);

        // Complete the job
        let completion = JobCompletion {
            output_size,
            output_bitrate,
        };
        client.complete_job(&job_id, completion).await?;

        log::info!("Job {} completed successfully", job_id);

        // Clean up temp files
        let _ = tokio::fs::remove_file(&input_path).await;
        let _ = tokio::fs::remove_file(&output_path).await;

        // Update state
        {
            let mut state = self.state.lock().await;
            state.current_job_id = None;
        }

        // Emit job completed event
        let _ = self.app_handle.emit("job-completed", &job_id);

        Ok(true)
    }
}
