use super::{ActorCommand, ActorError, ActorResult, ClientCommand, InitializationStatus, JobCommand};
use crate::clients::models::{Client, ClientStatus};
use crate::jobs::models::{EncodingJob, ProgressUpdate};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

/// Timeout for actor commands (30 seconds)
const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

/// Handle for communicating with the job management actor
#[derive(Clone)]
pub struct JobActorHandle {
    sender: mpsc::Sender<ActorCommand>,
}

impl JobActorHandle {
    /// Create a new handle
    pub fn new(sender: mpsc::Sender<ActorCommand>) -> Self {
        Self { sender }
    }

    // ========== Job Operations ==========

    /// Create a new encoding job
    #[allow(dead_code)]
    pub async fn create_job(
        &self,
        media_file_path: String,
        priority: i64,
    ) -> ActorResult<EncodingJob> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::CreateJob {
                media_file_path,
                priority,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Atomically claim the next pending job for a client
    pub async fn claim_next_job(&self, client_id: String) -> ActorResult<Option<EncodingJob>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::ClaimNextJob {
                client_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Mark a job as started
    pub async fn start_job(&self, job_id: String) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::StartJob {
                job_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Update job progress
    pub async fn update_progress(
        &self,
        job_id: String,
        progress: ProgressUpdate,
    ) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::UpdateProgress {
                job_id,
                progress,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Update job phase (downloading, encoding, uploading)
    pub async fn update_phase(&self, job_id: String, phase: String) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::UpdatePhase {
                job_id,
                phase,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Complete a job
    pub async fn complete_job(
        &self,
        job_id: String,
        output_path: String,
        output_size: i64,
        output_bitrate: i64,
        average_speed: f64,
    ) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::CompleteJob {
                job_id,
                output_path,
                output_size,
                output_bitrate,
                average_speed,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Mark a job as failed
    pub async fn fail_job(&self, job_id: String, error_message: String) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::FailJob {
                job_id,
                error_message,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Requeue a job (cancel and reset)
    pub async fn requeue_job(&self, job_id: String) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::RequeueJob {
                job_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get job by ID
    pub async fn get_job(&self, job_id: String) -> ActorResult<Option<EncodingJob>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::GetJob {
                job_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get all active jobs
    pub async fn get_active_jobs(&self) -> ActorResult<Vec<EncodingJob>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::GetActiveJobs { respond_to: tx }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get count of pending jobs
    pub async fn get_pending_count(&self) -> ActorResult<i64> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::GetPendingCount {
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get count of active jobs
    pub async fn get_active_count(&self) -> ActorResult<i64> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::GetActiveCount {
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get frame count for a media file
    pub async fn get_media_file_frames(
        &self,
        media_file_path: String,
    ) -> ActorResult<Option<i64>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Job(JobCommand::GetMediaFileFrames {
                media_file_path,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    // ========== Client Operations ==========

    /// Register a new client or reconnect existing one
    pub async fn register_client(
        &self,
        display_name: String,
        computer_name: String,
        machine_id: Option<String>,
    ) -> ActorResult<Client> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(ClientCommand::RegisterClient {
                display_name,
                computer_name,
                machine_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Update client heartbeat timestamp
    pub async fn update_heartbeat(&self, client_id: String) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(ClientCommand::UpdateHeartbeat {
                client_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Disconnect a client
    #[allow(dead_code)]
    pub async fn disconnect_client(&self, client_id: String) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(ClientCommand::DisconnectClient {
                client_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Disconnect a client and requeue all their jobs
    pub async fn disconnect_and_requeue_jobs(&self, client_id: String) -> ActorResult<()> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(
                ClientCommand::DisconnectAndRequeueJobs {
                    client_id,
                    respond_to: tx,
                },
            ))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get client by ID
    #[allow(dead_code)]
    pub async fn get_client(&self, client_id: String) -> ActorResult<Option<Client>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(ClientCommand::GetClient {
                client_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get all connected clients
    pub async fn get_connected_clients(&self) -> ActorResult<Vec<Client>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(ClientCommand::GetConnectedClients {
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Check if a client is connected
    pub async fn is_client_connected(&self, client_id: String) -> ActorResult<bool> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(ClientCommand::IsClientConnected {
                client_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get client status with active job count
    #[allow(dead_code)]
    pub async fn get_client_status(
        &self,
        client_id: String,
    ) -> ActorResult<Option<ClientStatus>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(ClientCommand::GetClientStatus {
                client_id,
                respond_to: tx,
            }))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    /// Get all client statuses
    pub async fn get_all_client_statuses(&self) -> ActorResult<Vec<ClientStatus>> {
        let (tx, rx) = oneshot::channel();

        self.sender
            .send(ActorCommand::Client(
                ClientCommand::GetAllClientStatuses { respond_to: tx },
            ))
            .await
            .map_err(|_| ActorError::InvalidState("Actor not running".to_string()))?;

        tokio::time::timeout(COMMAND_TIMEOUT, rx)
            .await
            .map_err(|_| ActorError::InvalidState("Command timeout".to_string()))?
            .map_err(|_| ActorError::InvalidState("Response channel closed".to_string()))?
    }

    // ========== Utility Operations ==========

    /// Get initialization status (no timeout, always succeeds)
    pub async fn get_initialization_status(&self) -> InitializationStatus {
        let (tx, rx) = oneshot::channel();

        if self
            .sender
            .send(ActorCommand::GetInitializationStatus { respond_to: tx })
            .await
            .is_err()
        {
            return InitializationStatus::Failed {
                error: "Actor not running".to_string(),
            };
        }

        rx.await.unwrap_or(InitializationStatus::Failed {
            error: "Response channel closed".to_string(),
        })
    }
}
