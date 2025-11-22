use super::models::{EncodingJob, EncodingProgress, JobStatus, ProgressUpdate};
use anyhow::{anyhow, Result};
use sqlx::{Executor, SqlitePool};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Job queue manager
pub struct JobQueue {
    pool: SqlitePool,
    // In-memory cache for faster lookups (optional optimization)
    _cache: Arc<RwLock<Vec<EncodingJob>>>,
}

impl JobQueue {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            _cache: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a new job
    pub async fn create_job(&self, media_file_path: String, priority: i64) -> Result<EncodingJob> {
        let job = EncodingJob::new(media_file_path, priority);

        sqlx::query(
            r#"INSERT INTO encoding_jobs
            (id, media_file_path, status, priority, created_at)
            VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&job.id)
        .bind(&job.media_file_path)
        .bind(&job.status)
        .bind(job.priority)
        .bind(job.created_at)
        .execute(&self.pool)
        .await?;

        Ok(job)
    }

    /// Get next pending job with highest priority
    pub async fn get_next_job(&self) -> Result<Option<EncodingJob>> {
        let job: Option<EncodingJob> = sqlx::query_as(
            r#"SELECT * FROM encoding_jobs
            WHERE status = 'pending'
            ORDER BY priority DESC, created_at ASC
            LIMIT 1"#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(job)
    }

    /// Assign job to a client
    pub async fn assign_job(&self, job_id: &str, client_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, assigned_client = ?, assigned_at = ?
            WHERE id = ? AND status = 'pending'"#,
        )
        .bind(JobStatus::Assigned.as_str())
        .bind(client_id)
        .bind(now)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Job not found or already assigned"));
        }

        Ok(())
    }

    /// Mark job as in progress
    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, started_at = ?
            WHERE id = ?"#,
        )
        .bind(JobStatus::InProgress.as_str())
        .bind(now)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update job progress
    pub async fn update_progress(&self, job_id: &str, update: ProgressUpdate) -> Result<()> {
        let progress = EncodingProgress::new(job_id.to_string(), update);

        sqlx::query(
            r#"INSERT OR REPLACE INTO encoding_progress
            (job_id, frame, fps, bitrate, speed, updated_at)
            VALUES (?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&progress.job_id)
        .bind(progress.frame)
        .bind(progress.fps)
        .bind(&progress.bitrate)
        .bind(&progress.speed)
        .bind(progress.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Complete a job
    pub async fn complete_job(
        &self,
        job_id: &str,
        output_path: String,
        output_size: i64,
        output_bitrate: i64,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, completed_at = ?, output_path = ?, output_size = ?, output_bitrate = ?
            WHERE id = ?"#,
        )
        .bind(JobStatus::Completed.as_str())
        .bind(now)
        .bind(output_path)
        .bind(output_size)
        .bind(output_bitrate)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        // Update media_files table
        sqlx::query(
            r#"UPDATE media_files
            SET processed = 1, size = ?, bit_rate = ?
            WHERE path = (SELECT media_file_path FROM encoding_jobs WHERE id = ?)"#,
        )
        .bind(output_size)
        .bind(output_bitrate)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Fail a job
    pub async fn fail_job(&self, job_id: &str, error_message: String) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        // Increment retry count
        sqlx::query(
            r#"UPDATE media_files
            SET retry_count = retry_count + 1
            WHERE path = (SELECT media_file_path FROM encoding_jobs WHERE id = ?)"#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, completed_at = ?, error_message = ?
            WHERE id = ?"#,
        )
        .bind(JobStatus::Failed.as_str())
        .bind(now)
        .bind(error_message)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Requeue a failed or stale job
    pub async fn requeue_job(&self, job_id: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, assigned_client = NULL, assigned_at = NULL, started_at = NULL, error_message = NULL
            WHERE id = ?"#,
        )
        .bind(JobStatus::Pending.as_str())
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get job by ID
    pub async fn get_job(&self, job_id: &str) -> Result<Option<EncodingJob>> {
        let job: Option<EncodingJob> =
            sqlx::query_as(r#"SELECT * FROM encoding_jobs WHERE id = ?"#)
                .bind(job_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(job)
    }

    /// Get all active jobs (assigned or in progress)
    pub async fn get_active_jobs(&self) -> Result<Vec<EncodingJob>> {
        let jobs: Vec<EncodingJob> = sqlx::query_as(
            r#"SELECT * FROM encoding_jobs
            WHERE status IN ('assigned', 'in_progress')
            ORDER BY assigned_at DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    /// Get jobs assigned to a specific client
    pub async fn get_client_jobs(&self, client_id: &str) -> Result<Vec<EncodingJob>> {
        let jobs: Vec<EncodingJob> = sqlx::query_as(
            r#"SELECT * FROM encoding_jobs
            WHERE assigned_client = ? AND status IN ('assigned', 'in_progress')"#,
        )
        .bind(client_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    /// Get stale jobs (assigned but not started within timeout)
    pub async fn get_stale_jobs(&self, timeout_seconds: i64) -> Result<Vec<EncodingJob>> {
        let cutoff = chrono::Utc::now().timestamp() - timeout_seconds;

        let jobs: Vec<EncodingJob> = sqlx::query_as(
            r#"SELECT * FROM encoding_jobs
            WHERE status IN ('assigned', 'in_progress')
            AND assigned_at < ?"#,
        )
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await?;

        Ok(jobs)
    }

    /// Get job progress
    pub async fn get_progress(&self, job_id: &str) -> Result<Option<EncodingProgress>> {
        let progress: Option<EncodingProgress> =
            sqlx::query_as(r#"SELECT * FROM encoding_progress WHERE job_id = ?"#)
                .bind(job_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(progress)
    }

    /// Get count of pending jobs
    pub async fn get_pending_count(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM encoding_jobs WHERE status = 'pending'"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    /// Get count of active jobs
    pub async fn get_active_count(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM encoding_jobs WHERE status IN ('assigned', 'in_progress')"#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }
}
