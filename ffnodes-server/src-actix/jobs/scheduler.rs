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

    /// Check for stale jobs and requeue them atomically
    async fn check_stale_jobs(&self) -> Result<()> {
        debug!("Checking for stale jobs");

        // Use atomic requeue operation to prevent race conditions
        let requeued_count = self.queue.requeue_stale_jobs(self.timeout_seconds).await?;

        if requeued_count > 0 {
            info!("Requeued {} stale jobs", requeued_count);
        }

        Ok(())
    }
}
