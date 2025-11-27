use anyhow::Result;
use reqwest::{Client, multipart};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeRequest {
    pub server_guid: String,
    pub display_name: String,
    pub computer_name: String,
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
        let url = format!("{}/api/handshake", self.base_url);
        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        let handshake_response: HandshakeResponse = response.json().await?;
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

    #[allow(dead_code)]
    pub async fn heartbeat(&self, client_id: &str) -> Result<()> {
        let url = format!("{}/api/heartbeat/{}", self.base_url, client_id);
        self.add_auth_header(self.client.post(&url)).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn download_input_file(&self, job_id: &str, output_path: &Path) -> Result<()> {
        let url = format!("{}/api/files/{}/input", self.base_url, job_id);
        let response = self.add_auth_header(self.client.get(&url)).send().await?.error_for_status()?;

        let bytes = response.bytes().await?;
        tokio::fs::write(output_path, bytes).await?;

        Ok(())
    }

    pub async fn upload_output_file(&self, job_id: &str, file_path: &Path) -> Result<()> {
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

        // Read file
        log::trace!("Opening file for reading...");
        let mut file = File::open(file_path).await.map_err(|e| {
            log::error!("Failed to open file: {:#}", e);
            e
        })?;

        let mut buffer = Vec::new();
        log::trace!("Reading file contents...");
        file.read_to_end(&mut buffer).await.map_err(|e| {
            log::error!("Failed to read file: {:#}", e);
            e
        })?;

        log::debug!("✓ File read successfully - size: {} bytes", buffer.len());

        let filename = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("output.mp4")
            .to_string();
        log::debug!("Filename for upload: {}", filename);

        // Create multipart form
        log::trace!("Creating multipart form...");
        let file_part = multipart::Part::bytes(buffer)
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

