use super::queue::JobQueue;
use anyhow::Result;
use log::{debug, info, warn};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;

/// Job scheduler for handling stale jobs and rebalancing
pub struct JobScheduler {
    queue: Arc<JobQueue>,
    timeout_seconds: i64,
    check_interval_seconds: u64,
}

impl JobScheduler {
    pub fn new(queue: Arc<JobQueue>, timeout_seconds: i64) -> Self {
        Self {
            queue,
            timeout_seconds,
            check_interval_seconds: 60, // Check every minute
        }
    }

    /// Start the scheduler background task
    pub fn start(self: Arc<Self>) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            info!("Job scheduler started");
            let mut check_interval = interval(Duration::from_secs(self.check_interval_seconds));

            loop {
                check_interval.tick().await;

                if let Err(e) = self.check_stale_jobs().await {
                    warn!("Error checking stale jobs: {:#}", e);
                }
            }
        })
    }

    /// Check for stale jobs and requeue them
    async fn check_stale_jobs(&self) -> Result<()> {
        debug!("Checking for stale jobs");

        let stale_jobs = self.queue.get_stale_jobs(self.timeout_seconds).await?;

        if !stale_jobs.is_empty() {
            info!("Found {} stale jobs, requeuing...", stale_jobs.len());

            for job in stale_jobs {
                info!("Requeuing stale job: {} (assigned to: {:?})",
                    job.id, job.assigned_client);

                if let Err(e) = self.queue.requeue_job(&job.id).await {
                    warn!("Failed to requeue job {}: {:#}", job.id, e);
                } else {
                    debug!("Successfully requeued job: {}", job.id);
                }
            }
        }

        Ok(())
    }
}
