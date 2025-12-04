use super::models::{EncodingJob, EncodingProgress, JobStatus, ProgressUpdate};
use anyhow::{Result, anyhow, Context};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

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
        let job = EncodingJob::new(media_file_path.clone(), priority);

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
        .await
        .context(format!("Failed to insert job into database: job_id={}, path={}", job.id, media_file_path))?;

        info!("Created new encoding job: job_id={}, path={}, priority={}", job.id, media_file_path, priority);
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
        .await
        .context("Failed to query next pending job from database")?;

        if let Some(ref job) = job {
            debug!("Found next pending job: job_id={}, priority={}", job.id, job.priority);
        }
        Ok(job)
    }

    /// Atomically get and assign next pending job to a client
    /// This prevents TOCTOU race conditions by combining get + assign in a single atomic operation
    pub async fn claim_next_job(&self, client_id: &str) -> Result<Option<EncodingJob>> {
        let now = chrono::Utc::now().timestamp();

        // Use a transaction to ensure atomicity
        let mut tx = self.pool.begin().await
            .context("Failed to begin database transaction for job claiming")?;

        // Get the next pending job, excluding already processed files
        let job: Option<EncodingJob> = sqlx::query_as(
            r#"SELECT ej.* FROM encoding_jobs ej
            INNER JOIN media_files mf ON ej.media_file_path = mf.path
            WHERE ej.status = 'pending' AND mf.processed = 0
            ORDER BY ej.priority DESC, ej.created_at ASC
            LIMIT 1"#,
        )
        .fetch_optional(&mut *tx)
        .await
        .context("Failed to query next pending job in transaction")?;

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
            .await
            .context(format!("Failed to update job assignment: job_id={}, client_id={}", job.id, client_id))?;

            if result.rows_affected() == 0 {
                // Job was claimed by another process between SELECT and UPDATE
                warn!("Job claim race condition detected: job_id={}, already claimed by another client", job.id);
                tx.rollback().await
                    .context(format!("Failed to rollback transaction after race condition: job_id={}", job.id))?;
                return Ok(None);
            }

            tx.commit().await
                .context(format!("Failed to commit job claim transaction: job_id={}, client_id={}", job.id, client_id))?;

            info!("Job claimed atomically: job_id={}, client_id={}", job.id, client_id);

            // Return the updated job
            Ok(Some(EncodingJob {
                status: JobStatus::Assigned.as_str().to_string(),
                assigned_client: Some(client_id.to_string()),
                assigned_at: Some(now),
                ..job
            }))
        } else {
            debug!("No pending jobs available for claiming");
            tx.rollback().await
                .context("Failed to rollback transaction when no jobs available")?;
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
        .await
        .context(format!("Failed to assign job to client: job_id={}, client_id={}", job_id, client_id))?;

        if result.rows_affected() == 0 {
            error!("Job assignment failed - job not found or already assigned: job_id={}", job_id);
            return Err(anyhow!("Job not found or already assigned"));
        }

        info!("Job assigned to client: job_id={}, client_id={}", job_id, client_id);
        Ok(())
    }

    /// Mark job as in progress
    pub async fn start_job(&self, job_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        // Use transaction for atomicity
        let mut tx = self.pool.begin().await
            .context(format!("Failed to begin transaction for start_job: job_id={}", job_id))?;

        // Get current status within transaction
        let current_status: Option<(String,)> = sqlx::query_as(
            "SELECT status FROM encoding_jobs WHERE id = ?"
        )
        .bind(job_id)
        .fetch_optional(&mut *tx)
        .await
        .context(format!("Failed to query current job status: job_id={}", job_id))?;

        let status = match current_status {
            Some((s,)) => {
                info!("Starting job: job_id={}, current_status={}", job_id, s);
                s
            }
            None => {
                warn!("Cannot start job - not found: job_id={}", job_id);
                return Err(anyhow::anyhow!("Job not found: {}", job_id));
            }
        };

        // Validate status before updating
        if status != "assigned" {
            warn!("Cannot start job - invalid status: job_id={}, status={}", job_id, status);
            return Err(anyhow::anyhow!(
                "Job cannot be started from status '{}' (must be 'assigned')", status
            ));
        }

        // Update with status check in WHERE clause
        let result = sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, started_at = ?
            WHERE id = ? AND status = 'assigned'"#,
        )
        .bind(JobStatus::InProgress.as_str())
        .bind(now)
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .context(format!("Failed to mark job as in progress: job_id={}", job_id))?;

        if result.rows_affected() == 0 {
            error!("Job start failed - status changed during update: job_id={}", job_id);
            tx.rollback().await?;
            return Err(anyhow::anyhow!(
                "Failed to start job - status changed (race condition detected)"
            ));
        }

        // Commit transaction
        tx.commit().await
            .context(format!("Failed to commit start_job transaction: job_id={}", job_id))?;

        info!("✓ Job status updated to 'in_progress': job_id={}", job_id);
        Ok(())
    }

    /// Update job progress
    pub async fn update_progress(&self, job_id: &str, update: ProgressUpdate) -> Result<()> {
        let progress = EncodingProgress::new(job_id.to_string(), update.clone());

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
        .await
        .context(format!("Failed to update job progress: job_id={}, frame={}", job_id, update.frame))?;

        debug!("Progress updated: job_id={}, frame={}, fps={:.2}", job_id, update.frame, update.fps);
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
        .await
        .context(format!("Failed to update job phase: job_id={}, phase={}", job_id, phase))?;

        debug!("Phase updated: job_id={}, phase={}", job_id, phase);
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
        let mut tx = self.pool.begin().await
            .context(format!("Failed to begin transaction for job completion: job_id={}", job_id))?;

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
        .await
        .context(format!("Failed to update encoding_jobs table for completion: job_id={}", job_id))?;

        if result.rows_affected() == 0 {
            error!("Job completion failed - job not found: job_id={}", job_id);
            tx.rollback().await
                .context(format!("Failed to rollback transaction after job not found: job_id={}", job_id))?;
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
        .await
        .context(format!("Failed to update media_files table for completion: job_id={}", job_id))?;

        // Validate that the media file was actually updated
        if media_result.rows_affected() == 0 {
            error!("Job completion failed - media file not found: job_id={}", job_id);
            tx.rollback().await
                .context(format!("Failed to rollback transaction after media file not found: job_id={}", job_id))?;
            return Err(anyhow!("Media file not found for job {}", job_id));
        }

        // Commit both updates together
        tx.commit().await
            .context(format!("Failed to commit job completion transaction: job_id={}", job_id))?;

        info!("Job completed: job_id={}, output_size={}, output_bitrate={}", job_id, output_size, output_bitrate);
        Ok(())
    }

    /// Fail a job
    pub async fn fail_job(&self, job_id: &str, error_message: String) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        // Use transaction for atomicity
        let mut tx = self.pool.begin().await
            .context(format!("Failed to begin transaction for fail_job: job_id={}", job_id))?;

        // Increment retry count
        sqlx::query(
            r#"UPDATE media_files
            SET retry_count = retry_count + 1
            WHERE path = (SELECT media_file_path FROM encoding_jobs WHERE id = ?)"#,
        )
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .context(format!("Failed to increment retry count for failed job: job_id={}", job_id))?;

        // Update job status
        sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, completed_at = ?, error_message = ?
            WHERE id = ?"#,
        )
        .bind(JobStatus::Failed.as_str())
        .bind(now)
        .bind(&error_message)
        .bind(job_id)
        .execute(&mut *tx)
        .await
        .context(format!("Failed to mark job as failed: job_id={}, error={}", job_id, error_message))?;

        // Commit transaction
        tx.commit().await
            .context(format!("Failed to commit fail_job transaction: job_id={}", job_id))?;

        warn!("Job failed: job_id={}, error={}", job_id, error_message);
        Ok(())
    }

    /// Requeue a failed or stale job
    pub async fn requeue_job(&self, job_id: &str) -> Result<()> {
        let result = sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = ?, assigned_client = NULL, assigned_at = NULL, started_at = NULL, error_message = NULL
            WHERE id = ?"#,
        )
        .bind(JobStatus::Pending.as_str())
        .bind(job_id)
        .execute(&self.pool)
        .await
        .context(format!("Failed to requeue job: job_id={}", job_id))?;

        if result.rows_affected() > 0 {
            info!("Job requeued: job_id={}", job_id);
        } else {
            warn!("Job requeue failed - job not found: job_id={}", job_id);
        }

        Ok(())
    }

    /// Get job by ID
    pub async fn get_job(&self, job_id: &str) -> Result<Option<EncodingJob>> {
        let job: Option<EncodingJob> =
            sqlx::query_as(r#"SELECT * FROM encoding_jobs WHERE id = ?"#)
                .bind(job_id)
                .fetch_optional(&self.pool)
                .await
                .context(format!("Failed to query job by ID: job_id={}", job_id))?;

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
        .await
        .context("Failed to query active jobs from database")?;

        debug!("Retrieved {} active jobs", jobs.len());
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
        .await
        .context(format!("Failed to query client jobs: client_id={}", client_id))?;

        debug!("Retrieved {} jobs for client: {}", jobs.len(), client_id);
        Ok(jobs)
    }

    /// Atomically requeue all stale jobs (assigned/in_progress but not updated within timeout)
    /// Returns the number of jobs requeued
    pub async fn requeue_stale_jobs(&self, timeout_seconds: i64) -> Result<u64> {
        let cutoff = chrono::Utc::now().timestamp() - timeout_seconds;

        // Atomically update all stale jobs in a single query
        // A job is stale if it was assigned before the cutoff AND
        // (the assigned client is dead/missing OR the client hasn't sent a heartbeat since the cutoff)
        let result = sqlx::query(
            r#"UPDATE encoding_jobs
            SET status = 'pending', assigned_client = NULL, assigned_at = NULL, started_at = NULL, error_message = NULL
            WHERE status IN ('assigned', 'in_progress')
            AND assigned_at < ?
            AND (
                assigned_client IS NULL
                OR
                assigned_client IN (SELECT id FROM clients WHERE last_heartbeat < ?)
            )"#,
        )
        .bind(cutoff)
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .context(format!("Failed to requeue stale jobs: timeout_seconds={}", timeout_seconds))?;

        let count = result.rows_affected();
        if count > 0 {
            warn!("Requeued {} stale jobs (timeout: {}s)", count, timeout_seconds);
        }
        Ok(count)
    }

    /// Get stale jobs (assigned but not started within timeout)
    pub async fn get_stale_jobs(&self, timeout_seconds: i64) -> Result<Vec<EncodingJob>> {
        let cutoff = chrono::Utc::now().timestamp() - timeout_seconds;

        let jobs: Vec<EncodingJob> = sqlx::query_as(
            r#"SELECT * FROM encoding_jobs
            WHERE status IN ('assigned', 'in_progress')
            AND assigned_at < ?
            AND (
                assigned_client IS NULL
                OR
                assigned_client IN (SELECT id FROM clients WHERE last_heartbeat < ?)
            )"#,
        )
        .bind(cutoff)
        .bind(cutoff)
        .fetch_all(&self.pool)
        .await
        .context(format!("Failed to query stale jobs: timeout_seconds={}", timeout_seconds))?;

        if !jobs.is_empty() {
            debug!("Found {} stale jobs", jobs.len());
        }
        Ok(jobs)
    }

    /// Get job progress
    #[allow(dead_code)]
    pub async fn get_progress(&self, job_id: &str) -> Result<Option<EncodingProgress>> {
        let progress: Option<EncodingProgress> =
            sqlx::query_as(r#"SELECT * FROM encoding_progress WHERE job_id = ?"#)
                .bind(job_id)
                .fetch_optional(&self.pool)
                .await
                .context(format!("Failed to query job progress: job_id={}", job_id))?;

        Ok(progress)
    }

    /// Get count of pending jobs
    pub async fn get_pending_count(&self) -> Result<i64> {
        let count: (i64,) =
            sqlx::query_as(r#"SELECT COUNT(*) FROM encoding_jobs WHERE status = 'pending'"#)
                .fetch_one(&self.pool)
                .await
                .context("Failed to count pending jobs")?;

        Ok(count.0)
    }

    /// Get count of active jobs
    pub async fn get_active_count(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM encoding_jobs WHERE status IN ('assigned', 'in_progress')"#,
        )
        .fetch_one(&self.pool)
        .await
        .context("Failed to count active jobs")?;

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
        .await
        .context("Failed to query unprocessed media files for job creation")?;

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
                .await
                .context(format!("Failed to query frame count for media file: path={}", media_file_path))?;

        Ok(frames.map(|f| f.0))
    }
}
