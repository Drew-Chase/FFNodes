use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Job status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Assigned => "assigned",
            JobStatus::InProgress => "in_progress",
            JobStatus::Completed => "completed",
            JobStatus::Failed => "failed",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(JobStatus::Pending),
            "assigned" => Some(JobStatus::Assigned),
            "in_progress" => Some(JobStatus::InProgress),
            "completed" => Some(JobStatus::Completed),
            "failed" => Some(JobStatus::Failed),
            _ => None,
        }
    }
}

/// Encoding job record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EncodingJob {
    pub id: String,
    pub media_file_path: String,
    pub status: String, // Stored as string in DB
    pub priority: i64,
    pub assigned_client: Option<String>,
    pub assigned_at: Option<i64>,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
    pub output_path: Option<String>,
    pub output_size: Option<i64>,
    pub output_bitrate: Option<i64>,
    pub average_speed: Option<f64>,
    pub current_phase: Option<String>, // "downloading", "encoding", "uploading"
    pub created_at: i64,
}

impl EncodingJob {
    pub fn new(media_file_path: String, priority: i64) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            media_file_path,
            status: JobStatus::Pending.as_str().to_string(),
            priority,
            assigned_client: None,
            assigned_at: None,
            started_at: None,
            completed_at: None,
            error_message: None,
            output_path: None,
            output_size: None,
            output_bitrate: None,
            average_speed: None,
            current_phase: None,
            created_at: now,
        }
    }

    #[allow(dead_code)]
    pub fn get_status(&self) -> JobStatus {
        JobStatus::from_str(&self.status).unwrap_or(JobStatus::Pending)
    }

    #[allow(dead_code)]
    pub fn set_status(&mut self, status: JobStatus) {
        self.status = status.as_str().to_string();
    }
}

/// Job request from client
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct JobRequest {
    pub client_id: String,
}

/// Job response to client
#[derive(Debug, Serialize, Deserialize)]
pub struct JobResponse {
    pub job: EncodingJob,
    pub input_path: String,
    pub output_template: String,
    pub total_frames: Option<i64>,
    pub ffmpeg_template: String,
    pub output_container: String,
}

/// Progress update from client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub frame: i64,
    pub fps: f64,
    pub bitrate: String,
    pub speed: String,
}

/// Job completion payload
#[derive(Debug, Serialize, Deserialize)]
pub struct JobCompletion {
    pub output_size: i64,
    pub output_bitrate: i64,
    pub average_speed: f64,
}

/// Job failure payload
#[derive(Debug, Serialize, Deserialize)]
pub struct JobFailure {
    pub error: String,
}

/// Encoding progress record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EncodingProgress {
    pub job_id: String,
    pub frame: i64,
    pub fps: f64,
    pub bitrate: String,
    pub speed: String,
    pub updated_at: i64,
}

impl EncodingProgress {
    pub fn new(job_id: String, update: ProgressUpdate) -> Self {
        Self {
            job_id,
            frame: update.frame,
            fps: update.fps,
            bitrate: update.bitrate,
            speed: update.speed,
            updated_at: chrono::Utc::now().timestamp(),
        }
    }
}
