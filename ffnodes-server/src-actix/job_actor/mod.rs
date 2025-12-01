use crate::clients::models::{Client, ClientStatus};
use crate::jobs::models::{EncodingJob, ProgressUpdate};
use tokio::sync::oneshot;

pub mod actor;
pub mod handle;

pub use actor::JobActor;
pub use handle::JobActorHandle;

/// Result type for actor operations
pub type ActorResult<T> = Result<T, ActorError>;

/// Actor error types
#[derive(Debug, Clone)]
pub enum ActorError {
    /// Actor is not yet initialized
    NotInitialized,
    /// Database operation failed
    DatabaseError(String),
    /// Resource not found
    NotFound,
    /// Invalid state or operation
    InvalidState(String),
}

impl std::fmt::Display for ActorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActorError::NotInitialized => write!(f, "Actor not initialized"),
            ActorError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ActorError::NotFound => write!(f, "Resource not found"),
            ActorError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
        }
    }
}

impl std::error::Error for ActorError {}

/// Initialization status of the actor
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InitializationStatus {
    /// Actor has not started initialization
    NotStarted,
    /// Actor is currently initializing
    InProgress { stage: String },
    /// Actor initialization is complete
    Complete,
    /// Actor initialization failed
    Failed { error: String },
}

/// Job-related commands
#[derive(Debug)]
pub enum JobCommand {
    /// Create a new encoding job
    CreateJob {
        media_file_path: String,
        priority: i64,
        respond_to: oneshot::Sender<ActorResult<EncodingJob>>,
    },
    /// Atomically claim the next pending job for a client
    ClaimNextJob {
        client_id: String,
        respond_to: oneshot::Sender<ActorResult<Option<EncodingJob>>>,
    },
    /// Mark a job as started
    StartJob {
        job_id: String,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Update job progress
    UpdateProgress {
        job_id: String,
        progress: ProgressUpdate,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Update job phase (downloading, encoding, uploading)
    UpdatePhase {
        job_id: String,
        phase: String,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Complete a job
    CompleteJob {
        job_id: String,
        output_path: String,
        output_size: i64,
        output_bitrate: i64,
        average_speed: f64,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Mark a job as failed
    FailJob {
        job_id: String,
        error_message: String,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Requeue a job (cancel and reset)
    RequeueJob {
        job_id: String,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Get job by ID
    GetJob {
        job_id: String,
        respond_to: oneshot::Sender<ActorResult<Option<EncodingJob>>>,
    },
    /// Get all active jobs
    GetActiveJobs {
        respond_to: oneshot::Sender<ActorResult<Vec<EncodingJob>>>,
    },
    /// Get count of pending jobs
    GetPendingCount {
        respond_to: oneshot::Sender<ActorResult<i64>>,
    },
    /// Get count of active jobs
    GetActiveCount {
        respond_to: oneshot::Sender<ActorResult<i64>>,
    },
    /// Get frame count for a media file
    GetMediaFileFrames {
        media_file_path: String,
        respond_to: oneshot::Sender<ActorResult<Option<i64>>>,
    },
}

/// Client-related commands
#[derive(Debug)]
pub enum ClientCommand {
    /// Register a new client or reconnect existing one
    RegisterClient {
        display_name: String,
        computer_name: String,
        machine_id: Option<String>,
        respond_to: oneshot::Sender<ActorResult<Client>>,
    },
    /// Update client heartbeat timestamp
    UpdateHeartbeat {
        client_id: String,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Disconnect a client
    DisconnectClient {
        client_id: String,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Disconnect a client and requeue all their jobs
    DisconnectAndRequeueJobs {
        client_id: String,
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Get client by ID
    GetClient {
        client_id: String,
        respond_to: oneshot::Sender<ActorResult<Option<Client>>>,
    },
    /// Get all connected clients
    GetConnectedClients {
        respond_to: oneshot::Sender<ActorResult<Vec<Client>>>,
    },
    /// Check if a client is connected
    IsClientConnected {
        client_id: String,
        respond_to: oneshot::Sender<ActorResult<bool>>,
    },
    /// Get client status with active job count
    GetClientStatus {
        client_id: String,
        respond_to: oneshot::Sender<ActorResult<Option<ClientStatus>>>,
    },
    /// Get all client statuses
    GetAllClientStatuses {
        respond_to: oneshot::Sender<ActorResult<Vec<ClientStatus>>>,
    },
}

/// Top-level actor command
#[derive(Debug)]
pub enum ActorCommand {
    /// Job-related command
    Job(JobCommand),
    /// Client-related command
    Client(ClientCommand),
    /// Shutdown the actor
    Shutdown {
        respond_to: oneshot::Sender<ActorResult<()>>,
    },
    /// Get initialization status
    GetInitializationStatus {
        respond_to: oneshot::Sender<InitializationStatus>,
    },
}
