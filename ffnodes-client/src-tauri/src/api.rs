use anyhow::Result;
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use futures_util::{StreamExt, SinkExt};
use tokio_util::io::ReaderStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub server_guid: String,
    pub display_name: String,
    pub computer_name: String,
    pub machine_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeResponse {
    pub client_id: String,
    pub auth_token: String,
    pub ffmpeg_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingJob {
    pub id: String,
    pub media_file_path: String,
    pub status: String,
    pub priority: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResponse {
    pub job: EncodingJob,
    pub input_path: String,
    pub output_template: String,
    pub total_frames: Option<i64>,
    pub ffmpeg_template: String,
    pub output_container: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub frame: i64,
    pub fps: f64,
    pub bitrate: f64,
    pub speed: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobCompletion {
    pub output_size: i64,
    pub output_bitrate: i64,
    pub average_speed: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransferProgress {
    pub transferred_bytes: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub bytes_per_second: f64,
}

/// WebSocket event types matching server WsEvent enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    JobAssigned {
        job_id: String,
        client_id: String,
        media_file: String,
    },
    Progress {
        job_id: String,
        client_id: String,
        frame: i64,
        fps: f64,
        speed: String,
    },
    JobCompleted {
        job_id: String,
        client_id: String,
    },
    JobFailed {
        job_id: String,
        client_id: String,
        error: String,
    },
    ClientConnected {
        client_id: String,
        display_name: String,
    },
    ClientDisconnected {
        client_id: String,
    },
}

#[derive(Clone)]
pub struct ServerClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

impl ServerClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            auth_token: None,
        }
    }

    pub fn with_auth(base_url: String, auth_token: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            auth_token: Some(auth_token),
        }
    }

    fn add_auth_header(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(token) = &self.auth_token {
            builder.header("Authorization", format!("Bearer {}", token))
        } else {
            builder
        }
    }

    pub async fn handshake(&self, request: HandshakeRequest) -> Result<HandshakeResponse> {
        let url = format!("{}/api/auth/handshake", self.base_url);

        log::info!("=== Starting handshake ===");
        log::info!("Handshake URL: {}", url);
        log::info!("Base URL: {}", self.base_url);

        // Log request details
        log::debug!("Handshake request details:");
        log::debug!("  server_guid: {}", request.server_guid);
        log::debug!("  display_name: {}", request.display_name);
        log::debug!("  computer_name: {}", request.computer_name);
        log::debug!("  machine_id: {}", request.machine_id);

        // Serialize request to see exact JSON being sent
        match serde_json::to_string_pretty(&request) {
            Ok(json_str) => log::trace!("Request body JSON:\n{}", json_str),
            Err(e) => log::warn!("Failed to serialize request for logging: {}", e),
        }

        log::debug!("Sending POST request...");
        let response = match self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(resp) => {
                log::debug!("✓ Request sent successfully");
                resp
            }
            Err(e) => {
                log::error!("✗ Failed to send handshake request");
                log::error!("Error type: {:?}", e);
                log::error!("Error details: {:#}", e);

                // Check for common network errors
                if e.is_timeout() {
                    log::error!("→ Connection timed out - server may be unreachable");
                } else if e.is_connect() {
                    log::error!("→ Connection failed - check if server is running and URL is correct");
                } else if e.is_request() {
                    log::error!("→ Request construction failed - invalid URL or headers");
                }

                return Err(e.into());
            }
        };

        // Log response status and headers
        let status = response.status();
        log::info!("Response status: {} {}", status.as_u16(), status.canonical_reason().unwrap_or(""));
        log::trace!("Response headers: {:#?}", response.headers());

        // Check status code before reading body
        if !status.is_success() {
            log::error!("✗ Server returned error status: {}", status);

            // Try to read error body for more details
            match response.text().await {
                Ok(body) => {
                    log::error!("Error response body: {}", body);
                    return Err(anyhow::anyhow!("Handshake failed with status {}: {}", status, body));
                }
                Err(e) => {
                    log::error!("Failed to read error response body: {}", e);
                    return Err(anyhow::anyhow!("Handshake failed with status {}", status));
                }
            }
        }

        // Read response body as text first for logging
        log::debug!("Reading response body...");
        let response_text = match response.text().await {
            Ok(text) => {
                log::debug!("✓ Response body received ({} bytes)", text.len());
                log::trace!("Raw response body:\n{}", text);
                text
            }
            Err(e) => {
                log::error!("✗ Failed to read response body: {}", e);
                return Err(e.into());
            }
        };

        // Parse JSON response
        log::debug!("Parsing JSON response...");
        let handshake_response: HandshakeResponse = match serde_json::from_str(&response_text) {
            Ok(parsed) => {
                log::debug!("✓ Successfully parsed handshake response");
                parsed
            }
            Err(e) => {
                log::error!("✗ Failed to parse JSON response");
                log::error!("Parse error: {}", e);
                log::error!("Response text that failed to parse: {}", response_text);
                return Err(anyhow::anyhow!("Failed to parse handshake response: {}", e));
            }
        };

        log::info!("✓ Handshake successful!");
        log::info!("Client ID: {}", handshake_response.client_id);
        log::info!("Auth token: {}...", &handshake_response.auth_token.chars().take(10).collect::<String>());
        log::debug!("FFmpeg template: {}", handshake_response.ffmpeg_template);
        log::info!("=== Handshake complete ===");

        Ok(handshake_response)
    }

    pub async fn request_job(&self, client_id: &str) -> Result<Option<JobResponse>> {
        let url = format!("{}/api/jobs/request/{}", self.base_url, client_id);
        let response = self.add_auth_header(self.client.post(&url)).send().await?;

        if response.status() == 204 {
            // No content - no jobs available
            return Ok(None);
        }

        let job_response: JobResponse = response.error_for_status()?.json().await?;
        Ok(Some(job_response))
    }

    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let url = format!("{}/api/jobs/{}/start", self.base_url, job_id);
        self.add_auth_header(self.client.post(&url)).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn update_progress(&self, job_id: &str, progress: ProgressUpdate) -> Result<()> {
        let url = format!("{}/api/jobs/{}/progress", self.base_url, job_id);
        self.add_auth_header(
            self.client
                .post(&url)
                .json(&progress)
        ).send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn complete_job(&self, job_id: &str, completion: JobCompletion) -> Result<()> {
        let url = format!("{}/api/jobs/{}/complete", self.base_url, job_id);
        self.add_auth_header(
            self.client
                .post(&url)
                .json(&completion)
        ).send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn fail_job(&self, job_id: &str, error: String) -> Result<()> {
        let url = format!("{}/api/jobs/{}/fail", self.base_url, job_id);
        let body = serde_json::json!({ "error": error });
        self.add_auth_header(
            self.client
                .post(&url)
                .json(&body)
        ).send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn cancel_job(&self, job_id: &str) -> Result<()> {
        let url = format!("{}/api/jobs/{}/cancel", self.base_url, job_id);
        self.add_auth_header(self.client.post(&url))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn disconnect(&self, client_id: &str) -> Result<()> {
        let url = format!("{}/api/clients/{}/disconnect", self.base_url, client_id);
        self.add_auth_header(self.client.post(&url))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn heartbeat(&self, client_id: &str) -> Result<()> {
        let url = format!("{}/api/jobs/heartbeat/{}", self.base_url, client_id);
        self.add_auth_header(self.client.post(&url)).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn download_input_file<F>(
        &self,
        job_id: &str,
        output_path: &Path,
        mut progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(TransferProgress) + Send + 'static,
    {
        let url = format!("{}/api/files/{}/input", self.base_url, job_id);
        log::debug!("Starting file download for job {}", job_id);
        log::debug!("Download URL: {}", url);
        log::debug!("Output path: {:?}", output_path);

        let response = self.add_auth_header(self.client.get(&url)).send().await?.error_for_status()?;

        // Get total size from Content-Length header
        let total_bytes = response
            .content_length()
            .ok_or_else(|| anyhow::anyhow!("Missing Content-Length header"))?;
        log::debug!("Total download size: {} bytes", total_bytes);

        // Create output file
        let mut file = tokio::fs::File::create(output_path).await?;

        // Stream response body with progress tracking
        let mut stream = response.bytes_stream();
        let mut downloaded_bytes: u64 = 0;
        let mut last_update = std::time::Instant::now();
        let mut last_bytes = 0u64;
        let mut speed_ema = 0.0f64; // Exponential moving average for speed
        let start_time = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let chunk_size = chunk.len() as u64;

            // Write chunk to file FIRST - this is the critical operation
            file.write_all(&chunk).await?;

            // Update progress tracking
            downloaded_bytes += chunk_size;

            // Throttle progress updates to ~100ms intervals, but always update on completion
            let now = std::time::Instant::now();
            let elapsed_since_last = now.duration_since(last_update).as_secs_f64();
            let is_complete = downloaded_bytes >= total_bytes;

            if elapsed_since_last >= 0.1 || is_complete {
                let bytes_since_last = downloaded_bytes - last_bytes;
                let current_speed = if elapsed_since_last > 0.0 {
                    bytes_since_last as f64 / elapsed_since_last
                } else {
                    // For very fast downloads, calculate speed based on total time
                    let total_elapsed = now.duration_since(start_time).as_secs_f64();
                    if total_elapsed > 0.0 {
                        downloaded_bytes as f64 / total_elapsed
                    } else {
                        0.0
                    }
                };

                // Smooth speed with exponential moving average
                let alpha = 0.3;
                speed_ema = if speed_ema == 0.0 {
                    current_speed
                } else {
                    alpha * current_speed + (1.0 - alpha) * speed_ema
                };

                let percentage = (downloaded_bytes as f64 / total_bytes as f64) * 100.0;

                progress_callback(TransferProgress {
                    transferred_bytes: downloaded_bytes,
                    total_bytes,
                    percentage,
                    bytes_per_second: speed_ema,
                });

                last_update = now;
                last_bytes = downloaded_bytes;
            }
        }

        // Ensure file is fully written
        file.flush().await?;
        file.sync_all().await?;

        log::debug!("✓ Download complete: {} bytes", downloaded_bytes);

        // Verify we downloaded the expected amount
        if downloaded_bytes != total_bytes {
            log::error!("Download incomplete: expected {} bytes, got {} bytes", total_bytes, downloaded_bytes);
            return Err(anyhow::anyhow!("Incomplete download: expected {} bytes, got {} bytes", total_bytes, downloaded_bytes));
        }

        Ok(())
    }

    pub async fn upload_output_file<F>(
        &self,
        job_id: &str,
        file_path: &Path,
        progress_callback: F,
    ) -> Result<()>
    where
        F: FnMut(TransferProgress) + Send + 'static,
    {
        let url = format!("{}/api/files/{}/output", self.base_url, job_id);

        log::debug!("Starting file upload for job {}", job_id);
        log::debug!("Upload URL: {}", url);
        log::debug!("File path: {:?}", file_path);

        // Check if file exists
        if !file_path.exists() {
            log::error!("File does not exist: {:?}", file_path);
            return Err(anyhow::anyhow!("Output file not found at {:?}", file_path));
        }
        log::debug!("✓ File exists");

        // Get file size
        let metadata = tokio::fs::metadata(file_path).await?;
        let total_bytes = metadata.len();
        log::debug!("Total upload size: {} bytes", total_bytes);

        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("output.mp4")
            .to_string();
        log::debug!("Filename for upload: {}", filename);

        // Open file for streaming
        let file = File::open(file_path).await.map_err(|e| {
            log::error!("Failed to open file: {:#}", e);
            e
        })?;

        // For upload, we need to track progress differently since we're streaming the request body
        // We'll create a simple byte counter that gets updated as chunks are yielded
        let uploaded_bytes = std::sync::Arc::new(tokio::sync::Mutex::new(0u64));
        let last_update = std::sync::Arc::new(tokio::sync::Mutex::new(std::time::Instant::now()));
        let last_bytes = std::sync::Arc::new(tokio::sync::Mutex::new(0u64));
        let speed_ema = std::sync::Arc::new(tokio::sync::Mutex::new(0.0f64));
        let start_time = std::time::Instant::now();

        // Wrap callback in Arc<Mutex> for thread-safe sharing
        let callback = std::sync::Arc::new(tokio::sync::Mutex::new(progress_callback));

        let uploaded_clone = uploaded_bytes.clone();
        let last_update_clone = last_update.clone();
        let last_bytes_clone = last_bytes.clone();
        let speed_ema_clone = speed_ema.clone();
        let callback_clone = callback.clone();

        // Create reader stream
        let mut reader_stream = ReaderStream::new(file);

        let stream = async_stream::stream! {
            while let Some(chunk) = reader_stream.next().await {
                match chunk {
                    Ok(bytes) => {
                        let chunk_size = bytes.len() as u64;

                        // Update uploaded bytes
                        let current_uploaded = {
                            let mut uploaded = uploaded_clone.lock().await;
                            *uploaded += chunk_size;
                            *uploaded
                        };

                        // Check if we should emit progress
                        let now = std::time::Instant::now();
                        let should_update = {
                            let last = last_update_clone.lock().await;
                            let elapsed = now.duration_since(*last).as_secs_f64();
                            elapsed >= 0.1 || current_uploaded >= total_bytes
                        };

                        if should_update {
                            let elapsed_since_last = {
                                let last = last_update_clone.lock().await;
                                now.duration_since(*last).as_secs_f64()
                            };

                            let bytes_since_last = {
                                let last_b = last_bytes_clone.lock().await;
                                current_uploaded - *last_b
                            };

                            let current_speed = if elapsed_since_last > 0.0 {
                                bytes_since_last as f64 / elapsed_since_last
                            } else {
                                // For very fast uploads, use total time
                                let total_elapsed = now.duration_since(start_time).as_secs_f64();
                                if total_elapsed > 0.0 {
                                    current_uploaded as f64 / total_elapsed
                                } else {
                                    0.0
                                }
                            };

                            // Smooth speed with exponential moving average
                            let alpha = 0.3;
                            let speed = {
                                let mut ema = speed_ema_clone.lock().await;
                                if *ema == 0.0 {
                                    *ema = current_speed;
                                } else {
                                    *ema = alpha * current_speed + (1.0 - alpha) * *ema;
                                }
                                *ema
                            };

                            let percentage = (current_uploaded as f64 / total_bytes as f64) * 100.0;

                            // Call progress callback (non-blocking)
                            {
                                let mut cb = callback_clone.lock().await;
                                cb(TransferProgress {
                                    transferred_bytes: current_uploaded,
                                    total_bytes,
                                    percentage,
                                    bytes_per_second: speed,
                                });
                            }

                            *last_update_clone.lock().await = now;
                            *last_bytes_clone.lock().await = current_uploaded;
                        }

                        yield Ok::<_, std::io::Error>(bytes);
                    }
                    Err(e) => {
                        log::error!("Error reading upload chunk: {}", e);
                        yield Err(e);
                        break;
                    }
                }
            }
        };

        // Create multipart form with streaming body
        log::trace!("Creating multipart form with streaming body...");
        let file_part = multipart::Part::stream(reqwest::Body::wrap_stream(stream))
            .file_name(filename.clone())
            .mime_str("video/mp4")?;

        let form = multipart::Form::new().part("file", file_part);
        log::debug!("✓ Multipart form created");

        log::trace!("Sending POST request...");
        let response = self.add_auth_header(
            self.client
                .post(&url)
                .multipart(form)
        ).send()
            .await
            .map_err(|e| {
                log::error!("Upload request failed: {:#}", e);
                e
            })?;

        let status = response.status();
        log::debug!("Response status: {}", status);

        if !status.is_success() {
            let body = response.text().await.unwrap_or_else(|_| "Could not read response body".to_string());
            log::error!("Upload failed with status {}: {}", status, body);
            return Err(anyhow::anyhow!("Upload failed: {} - {}", status, body));
        }

        log::debug!("✓ Upload successful");
        Ok(())
    }

    pub async fn get_active_jobs(&self) -> Result<Vec<EncodingJob>> {
        let url = format!("{}/api/jobs/active", self.base_url);
        let response = self.add_auth_header(self.client.get(&url)).send().await?.error_for_status()?;
        let jobs: Vec<EncodingJob> = response.json().await?;
        Ok(jobs)
    }

    // Statistics endpoints
    pub async fn get_client_history(&self, client_id: &str) -> Result<ClientHistoryResponse> {
        let url = format!("{}/api/stats/client/{}/history", self.base_url, client_id);
        let response = self.add_auth_header(self.client.get(&url)).send().await?.error_for_status()?;
        let history: ClientHistoryResponse = response.json().await?;
        Ok(history)
    }

    pub async fn get_remote_progress(&self, exclude_client_id: Option<&str>) -> Result<RemoteProgressResponse> {
        let mut url = format!("{}/api/stats/remote-progress", self.base_url);
        if let Some(client_id) = exclude_client_id {
            url = format!("{}?exclude_self={}", url, client_id);
        }
        let response = self.add_auth_header(self.client.get(&url)).send().await?.error_for_status()?;
        let progress: RemoteProgressResponse = response.json().await?;
        Ok(progress)
    }

    pub async fn get_leaderboard(&self, category: &str) -> Result<LeaderboardResponse> {
        let url = format!("{}/api/stats/leaderboard?category={}", self.base_url, category);
        let response = self.add_auth_header(self.client.get(&url)).send().await?.error_for_status()?;
        let leaderboard: LeaderboardResponse = response.json().await?;
        Ok(leaderboard)
    }

    /// Connect to WebSocket for real-time progress updates
    /// Returns a receiver channel for incoming WebSocket events
    pub async fn connect_websocket(&self, client_id: &str) -> Result<mpsc::UnboundedReceiver<WsEvent>> {
        // Convert http/https URL to ws/wss
        let ws_url = self.base_url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let ws_url = format!("{}/api/ws/progress?client_id={}", ws_url, client_id);

        log::info!("Connecting to WebSocket: {}", ws_url);

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&ws_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to WebSocket: {}", e))?;

        log::info!("✓ WebSocket connected");

        let (mut write, mut read) = ws_stream.split();

        // Create channel for sending events to the application
        let (tx, rx) = mpsc::unbounded_channel();

        // Spawn task to handle incoming WebSocket messages
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Parse JSON message into WsEvent
                        match serde_json::from_str::<WsEvent>(&text) {
                            Ok(event) => {
                                log::debug!("Received WebSocket event: {:?}", event);
                                if tx.send(event).is_err() {
                                    log::warn!("Failed to send WebSocket event - receiver dropped");
                                    break;
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse WebSocket message: {} - {}", e, text);
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        log::trace!("Received WebSocket ping, sending pong");
                        if let Err(e) = write.send(Message::Pong(data)).await {
                            log::error!("Failed to send WebSocket pong: {}", e);
                            break;
                        }
                    }
                    Ok(Message::Close(reason)) => {
                        log::info!("WebSocket closed: {:?}", reason);
                        break;
                    }
                    Ok(_) => {
                        // Ignore other message types (binary, pong, etc.)
                    }
                    Err(e) => {
                        log::error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
            log::info!("WebSocket connection closed");
        });

        Ok(rx)
    }
}

// Statistics response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobHistoryEntry {
    pub filename: String,
    pub average_speed: f64,
    pub duration_seconds: i64,
    pub size_before: i64,
    pub size_after: i64,
    pub size_saved: i64,
    pub size_reduction_percent: f64,
    pub completed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallStats {
    pub average_speed: f64,
    pub average_duration_seconds: f64,
    pub average_size_reduction_percent: f64,
    pub total_jobs: i64,
    pub total_size_saved: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHistoryResponse {
    pub jobs: Vec<JobHistoryEntry>,
    pub overall: OverallStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteJobProgress {
    pub client_name: String,
    pub filename: String,
    pub percentage: f64,
    pub speed: f64,
    pub frame: i64,
    pub total_frames: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteProgressResponse {
    pub active_jobs: Vec<RemoteJobProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub client_name: String,
    pub value: f64,
    pub formatted_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub category: String,
    pub entries: Vec<LeaderboardEntry>,
}

