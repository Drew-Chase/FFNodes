use super::models::{EncodingJob, EncodingProgress, JobStatus, ProgressUpdate};
use anyhow::{Result, anyhow};
use sqlx::SqlitePool;
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

    /// Get next pending job with highest priority, excluding already processed files
    pub async fn get_next_job(&self) -> Result<Option<EncodingJob>> {
        let job: Option<EncodingJob> = sqlx::query_as(
            r#"SELECT ej.* FROM encoding_jobs ej
            INNER JOIN media_files mf ON ej.media_file_path = mf.path
            WHERE ej.status = 'pending' AND mf.processed = 0
            ORDER BY ej.priority DESC, ej.created_at ASC
            LIMIT 1"#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(job)
    }

    /// Atomically get and assign next pending job to a client
    /// This prevents TOCTOU race conditions by combining get + assign in a single atomic operation
    pub async fn claim_next_job(&self, client_id: &str) -> Result<Option<EncodingJob>> {
        let now = chrono::Utc::now().timestamp();

        // Use a transaction to ensure atomicity
        let mut tx = self.pool.begin().await?;

        // Get the next pending job, excluding already processed files
        let job: Option<EncodingJob> = sqlx::query_as(
            r#"SELECT ej.* FROM encoding_jobs ej
            INNER JOIN media_files mf ON ej.media_file_path = mf.path
            WHERE ej.status = 'pending' AND mf.processed = 0
            ORDER BY ej.priority DESC, ej.created_at ASC
            LIMIT 1"#,
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(job) = job {
            // Immediately assign it in the same transaction
            let result = sqlx::query(
                r#"UPDATE encoding_jobs
                SET status = ?, assigned_client = ?, assigned_at = ?
                WHERE id = ? AND status = 'pending'"#,
            )
            .bind(JobStatus::Assigned.as_str())
            .bind(client_id)
            .bind(now)
            .bind(&job.id)
            .execute(&mut *tx)
            .await?;

            if result.rows_affected() == 0 {
                // Job was claimed by another process between SELECT and UPDATE
                tx.rollback().await?;
                return Ok(None);
            }

            tx.commit().await?;

            // Return the updated job
            Ok(Some(EncodingJob {
                status: JobStatus::Assigned.as_str().to_string(),
                assigned_client: Some(client_id.to_string()),
                assigned_at: Some(now),
                ..job
            }))
        } else {
            tx.rollback().await?;
            Ok(None)
        }
    }

    /// Assign job to a client (legacy method, prefer claim_next_job for atomic operations)
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

    /// Update job's current phase
    pub async fn update_phase(&self, job_id: &str, phase: &str) -> Result<()> {
        sqlx::query(
            r#"UPDATE encoding_jobs
            SET current_phase = ?
            WHERE id = ?"#,
        )
        .bind(phase)
        .bind(job_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Complete a job with transaction isolation
    /// Both encoding_jobs and media_files updates are atomic
    pub async fn complete_job(
        &self,
        job_id: &str,
        output_path: String,
        output_size: i64,
        output_bitrate: i64,
        average_speed: f64,
    ) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        // Use a transaction to ensure both updates succeed or both fail
        let mut tx = self.pool.begin().await?;

        // Update encoding_jobs table
        let result = sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, completed_at = ?, output_path = ?, output_size = ?, output_bitrate = ?, average_speed = ?
            WHERE id = ?"#,
        )
        .bind(JobStatus::Completed.as_str())
        .bind(now)
        .bind(&output_path)
        .bind(output_size)
        .bind(output_bitrate)
        .bind(average_speed)
        .bind(job_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(anyhow!("Job not found"));
        }

        // Update media_files table - mark as processed and update metrics
        let media_result = sqlx::query(
            r#"UPDATE media_files
            SET processed = 1, size = ?, bit_rate = ?
            WHERE path = (SELECT media_file_path FROM encoding_jobs WHERE id = ?)"#,
        )
        .bind(output_size)
        .bind(output_bitrate)
        .bind(job_id)
        .execute(&mut *tx)
        .await?;

        // Validate that the media file was actually updated
        if media_result.rows_affected() == 0 {
            tx.rollback().await?;
            return Err(anyhow!("Media file not found for job {}", job_id));
        }

        // Commit both updates together
        tx.commit().await?;

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
    #[allow(dead_code)]
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

    /// Atomically requeue all stale jobs (assigned/in_progress but not updated within timeout)
    /// Returns the number of jobs requeued
    pub async fn requeue_stale_jobs(&self, timeout_seconds: i64) -> Result<u64> {
        let cutoff = chrono::Utc::now().timestamp() - timeout_seconds;

        // Atomically update all stale jobs in a single query
        let result = sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = 'pending', assigned_client = NULL, assigned_at = NULL, started_at = NULL, error_message = NULL
            WHERE status IN ('assigned', 'in_progress')
            AND assigned_at < ?"#,
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
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
    #[allow(dead_code)]
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
        let count: (i64,) =
            sqlx::query_as(r#"SELECT COUNT(*) FROM encoding_jobs WHERE status = 'pending'"#)
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

    /// Create jobs for media files that have been scanned but have no encoding job yet
    pub async fn create_jobs_for_unprocessed_files(&self) -> Result<usize> {
        // Query all media files that are not processed and have no associated job
        let files: Vec<(String, i64, i64)> = sqlx::query_as(
            r#"SELECT mf.path, mf.scanned_size, mf.encoding_complexity
            FROM media_files mf
            LEFT JOIN encoding_jobs ej ON mf.path = ej.media_file_path
            WHERE mf.processed = 0 AND ej.id IS NULL"#,
        )
        .fetch_all(&self.pool)
        .await?;

        let count = files.len();
        for (path, _size, complexity) in files {
            // Priority based on encoding complexity only (resolution × bitrate × duration)
            // Divide by 1000 to keep numbers manageable
            let priority = complexity / 1000;
            match self.create_job(path.clone(), priority).await {
                Ok(job) => {
                    tracing::info!("Created job {} for existing file: {}", job.id, path);
                }
                Err(e) => {
                    tracing::error!("Failed to create job for {}: {:#}", path, e);
                }
            }
        }

        Ok(count)
    }

    /// Get frame count for a media file
    pub async fn get_media_file_frames(&self, media_file_path: &str) -> Result<Option<i64>> {
        let frames: Option<(i64,)> =
            sqlx::query_as(r#"SELECT frames FROM media_files WHERE path = ?"#)
                .bind(media_file_path)
                .fetch_optional(&self.pool)
                .await?;

        Ok(frames.map(|f| f.0))
    }
}
