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
}

impl ServerClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
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
        let response = self.client.post(&url).send().await?;

        if response.status() == 204 {
            // No content - no jobs available
            return Ok(None);
        }

        let job_response: JobResponse = response.error_for_status()?.json().await?;
        Ok(Some(job_response))
    }

    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let url = format!("{}/api/jobs/{}/start", self.base_url, job_id);
        self.client.post(&url).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn update_progress(&self, job_id: &str, progress: ProgressUpdate) -> Result<()> {
        let url = format!("{}/api/jobs/{}/progress", self.base_url, job_id);
        self.client
            .post(&url)
            .json(&progress)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn complete_job(&self, job_id: &str, completion: JobCompletion) -> Result<()> {
        let url = format!("{}/api/jobs/{}/complete", self.base_url, job_id);
        self.client
            .post(&url)
            .json(&completion)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn fail_job(&self, job_id: &str, error: String) -> Result<()> {
        let url = format!("{}/api/jobs/{}/fail", self.base_url, job_id);
        let body = serde_json::json!({ "error": error });
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn heartbeat(&self, client_id: &str) -> Result<()> {
        let url = format!("{}/api/heartbeat/{}", self.base_url, client_id);
        self.client.post(&url).send().await?.error_for_status()?;
        Ok(())
    }

    pub async fn download_input_file(&self, job_id: &str, output_path: &Path) -> Result<()> {
        let url = format!("{}/api/files/{}/input", self.base_url, job_id);
        let response = self.client.get(&url).send().await?.error_for_status()?;

        let bytes = response.bytes().await?;
        tokio::fs::write(output_path, bytes).await?;

        Ok(())
    }

    pub async fn upload_output_file(&self, job_id: &str, file_path: &Path) -> Result<()> {
        let url = format!("{}/api/files/{}/output", self.base_url, job_id);

        // Read file
        let mut file = File::open(file_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        // Create multipart form
        let file_part = multipart::Part::bytes(buffer)
            .file_name(
                file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("output.mp4")
                    .to_string(),
            )
            .mime_str("video/mp4")?;

        let form = multipart::Form::new().part("file", file_part);

        self.client
            .post(&url)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    pub async fn get_active_jobs(&self) -> Result<Vec<EncodingJob>> {
        let url = format!("{}/api/jobs/active", self.base_url);
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let jobs: Vec<EncodingJob> = response.json().await?;
        Ok(jobs)
    }
}
