use super::{ActorCommand, ActorError, ClientCommand, InitializationStatus, JobCommand};
use crate::clients::ClientManager;
use crate::configuration::Configuration;
use crate::jobs::{JobQueue, JobScheduler};
use crate::media_files::{self, FileWatcher, Scanner};
use anyhow::Result;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Job management actor
pub struct JobActor {
    /// Database connection pool
    pool: Option<SqlitePool>,
    /// Job queue manager
    job_queue: Option<Arc<JobQueue>>,
    /// Client manager
    client_manager: Option<Arc<ClientManager>>,
    /// Server configuration
    config: Arc<Configuration>,
    /// Job scheduler background task handle
    scheduler_handle: Option<tokio::task::JoinHandle<()>>,
    /// Heartbeat monitor background task handle
    heartbeat_handle: Option<tokio::task::JoinHandle<()>>,
    /// File watcher
    file_watcher: Option<Arc<FileWatcher>>,
    /// Current initialization status
    initialization_status: InitializationStatus,
}

impl JobActor {
    /// Create a new job actor
    pub fn new(config: Arc<Configuration>, pool: Option<SqlitePool>) -> Self {
        Self {
            pool,
            job_queue: None,
            client_manager: None,
            config,
            scheduler_handle: None,
            heartbeat_handle: None,
            file_watcher: None,
            initialization_status: InitializationStatus::NotStarted,
        }
    }

    /// Run the actor main loop
    pub async fn run(mut self, mut receiver: mpsc::Receiver<ActorCommand>) {
        info!("Job management actor started");

        // Spawn initialization in background (non-blocking)
        let config = Arc::clone(&self.config);
        let pool = self.pool.clone();
        let init_handle = tokio::spawn(async move {
            Self::run_initialization(config, pool).await
        });

        // Wait for initialization to complete
        match init_handle.await {
            Ok(Ok((pool, job_queue, client_manager, scheduler_handle, heartbeat_handle, file_watcher))) => {
                self.pool = Some(pool);
                self.job_queue = Some(job_queue);
                self.client_manager = Some(client_manager);
                self.scheduler_handle = Some(scheduler_handle);
                self.heartbeat_handle = Some(heartbeat_handle);
                self.file_watcher = Some(file_watcher);
                self.initialization_status = InitializationStatus::Complete;
                info!("Job management actor initialization complete");
            }
            Ok(Err(e)) => {
                error!("Job management actor initialization failed: {:#}", e);
                self.initialization_status = InitializationStatus::Failed {
                    error: format!("{:#}", e),
                };
            }
            Err(e) => {
                error!("Job management actor initialization task panicked: {:#}", e);
                self.initialization_status = InitializationStatus::Failed {
                    error: format!("Initialization task panicked: {:#}", e),
                };
            }
        }

        // Main command loop
        while let Some(cmd) = receiver.recv().await {
            match cmd {
                ActorCommand::Shutdown { respond_to } => {
                    info!("Job management actor received shutdown command");
                    self.shutdown().await;
                    let _ = respond_to.send(Ok(()));
                    break;
                }
                ActorCommand::GetInitializationStatus { respond_to } => {
                    let _ = respond_to.send(self.initialization_status.clone());
                }
                ActorCommand::Job(job_cmd) => {
                    self.handle_job_command(job_cmd).await;
                }
                ActorCommand::Client(client_cmd) => {
                    self.handle_client_command(client_cmd).await;
                }
            }
        }

        info!("Job management actor shutting down");
    }

    /// Run initialization sequence (runs in background)
    async fn run_initialization(
        config: Arc<Configuration>,
        pool: Option<SqlitePool>,
    ) -> Result<(
        SqlitePool,
        Arc<JobQueue>,
        Arc<ClientManager>,
        tokio::task::JoinHandle<()>,
        tokio::task::JoinHandle<()>,
        Arc<FileWatcher>,
    )> {
        info!("Starting job management actor initialization");

        // Initialize database (if pool not provided)
        let pool = if let Some(pool) = pool {
            info!("Using provided database pool");
            pool
        } else {
            info!("Initializing database...");
            media_files::initialize().await?;
            let pool = media_files::media_file_db::open_pool().await?;
            info!("Database initialized successfully");
            pool
        };

        // Initialize job queue and client manager
        info!("Creating job queue and client manager...");
        let job_queue = Arc::new(JobQueue::new(pool.clone()));
        let client_manager = Arc::new(ClientManager::new(pool.clone()));
        info!("Job queue and client manager created");

        // Create jobs for existing unprocessed files
        info!("Checking for unprocessed media files...");
        match job_queue.create_jobs_for_unprocessed_files().await {
            Ok(count) => {
                if count > 0 {
                    info!("Created {} encoding jobs for existing unprocessed files", count);
                } else {
                    info!("No unprocessed files found");
                }
            }
            Err(e) => {
                error!("Failed to create jobs for unprocessed files: {:#}", e);
            }
        }

        // Start job scheduler
        info!("Starting job scheduler...");
        let job_scheduler = Arc::new(JobScheduler::new(
            Arc::clone(&job_queue),
            config.client_timeout_seconds as i64,
        ));
        let scheduler_handle = job_scheduler.start();
        info!("Job scheduler started");

        // Start heartbeat monitor
        info!("Starting heartbeat monitor...");
        let heartbeat_handle = Self::start_heartbeat_monitor(
            Arc::clone(&client_manager),
            Arc::clone(&job_queue),
        );
        info!("Heartbeat monitor started");

        // Start file watcher
        info!("Starting file watcher...");
        let file_watcher = Arc::new(FileWatcher::new(
            Arc::clone(&config),
            pool.clone(),
            Arc::clone(&job_queue),
        ));
        if let Err(e) = Arc::clone(&file_watcher).start().await {
            warn!("Failed to start file watcher: {:#}", e);
        } else {
            info!("File watcher started");
        }

        // Initial scan (spawn in background, don't wait for completion)
        info!("Starting initial media scan...");
        let watch_directories = config.watch_directories.clone();
        tokio::spawn({
            let config = Arc::clone(&config);
            let job_queue = Arc::clone(&job_queue);
            async move {
                if let Err(e) = Scanner::scan(watch_directories, config, job_queue).await {
                    error!("Media file scanner error: {}", e);
                } else {
                    info!("Initial media scan completed successfully");
                }
            }
        });

        Ok((
            pool,
            job_queue,
            client_manager,
            scheduler_handle,
            heartbeat_handle,
            file_watcher,
        ))
    }

    /// Start the heartbeat monitor background task
    fn start_heartbeat_monitor(
        client_manager: Arc<ClientManager>,
        job_queue: Arc<JobQueue>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;

                // Get all connected clients
                if let Ok(clients) = client_manager.get_connected_clients().await {
                    let now = chrono::Utc::now().timestamp();
                    let timeout_threshold = now - 60; // 60 seconds timeout

                    for client in clients {
                        // Check if client hasn't sent heartbeat in 60 seconds
                        if client.last_heartbeat < timeout_threshold {
                            info!(
                                "Client {} ({}) timed out (last heartbeat: {} seconds ago), disconnecting and requeuing jobs",
                                client.display_name,
                                client.id,
                                now - client.last_heartbeat
                            );

                            // Disconnect client and requeue their jobs
                            if let Err(e) = client_manager
                                .disconnect_and_requeue_jobs(&client.id, &job_queue)
                                .await
                            {
                                error!("Failed to disconnect stale client {}: {:#}", client.id, e);
                            }
                        }
                    }
                }
            }
        })
    }

    /// Handle job-related commands
    async fn handle_job_command(&self, cmd: JobCommand) {
        // Check if initialized
        if self.job_queue.is_none() {
            Self::send_not_initialized_error_job(cmd);
            return;
        }

        let job_queue = self.job_queue.as_ref().unwrap();

        match cmd {
            JobCommand::CreateJob {
                media_file_path,
                priority,
                respond_to,
            } => {
                let result = job_queue
                    .create_job(media_file_path, priority)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::ClaimNextJob {
                client_id,
                respond_to,
            } => {
                let result = job_queue
                    .claim_next_job(&client_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::StartJob {
                job_id,
                respond_to,
            } => {
                let result = job_queue
                    .start_job(&job_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::UpdateProgress {
                job_id,
                progress,
                respond_to,
            } => {
                let result = job_queue
                    .update_progress(&job_id, progress)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::UpdatePhase {
                job_id,
                phase,
                respond_to,
            } => {
                let result = job_queue
                    .update_phase(&job_id, &phase)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::CompleteJob {
                job_id,
                output_path,
                output_size,
                output_bitrate,
                average_speed,
                respond_to,
            } => {
                let result = job_queue
                    .complete_job(&job_id, output_path, output_size, output_bitrate, average_speed)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::FailJob {
                job_id,
                error_message,
                respond_to,
            } => {
                let result = job_queue
                    .fail_job(&job_id, error_message)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::RequeueJob {
                job_id,
                respond_to,
            } => {
                let result = job_queue
                    .requeue_job(&job_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::GetJob {
                job_id,
                respond_to,
            } => {
                let result = job_queue
                    .get_job(&job_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::GetActiveJobs { respond_to } => {
                let result = job_queue
                    .get_active_jobs()
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::GetPendingCount { respond_to } => {
                let result = job_queue
                    .get_pending_count()
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::GetActiveCount { respond_to } => {
                let result = job_queue
                    .get_active_count()
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            JobCommand::GetMediaFileFrames {
                media_file_path,
                respond_to,
            } => {
                let result = job_queue
                    .get_media_file_frames(&media_file_path)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
        }
    }

    /// Handle client-related commands
    async fn handle_client_command(&self, cmd: ClientCommand) {
        // Check if initialized
        if self.client_manager.is_none() || self.job_queue.is_none() {
            Self::send_not_initialized_error_client(cmd);
            return;
        }

        let client_manager = self.client_manager.as_ref().unwrap();
        let job_queue = self.job_queue.as_ref().unwrap();

        match cmd {
            ClientCommand::RegisterClient {
                display_name,
                computer_name,
                machine_id,
                respond_to,
            } => {
                let result = client_manager
                    .register_client(display_name, computer_name, machine_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            ClientCommand::UpdateHeartbeat {
                client_id,
                respond_to,
            } => {
                let result = client_manager
                    .update_heartbeat(&client_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            ClientCommand::DisconnectClient {
                client_id,
                respond_to,
            } => {
                let result = client_manager
                    .disconnect_client(&client_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            ClientCommand::DisconnectAndRequeueJobs {
                client_id,
                respond_to,
            } => {
                let result = client_manager
                    .disconnect_and_requeue_jobs(&client_id, job_queue)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            ClientCommand::GetClient {
                client_id,
                respond_to,
            } => {
                let result = client_manager
                    .get_client(&client_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            ClientCommand::GetConnectedClients { respond_to } => {
                let result = client_manager
                    .get_connected_clients()
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            ClientCommand::IsClientConnected {
                client_id,
                respond_to,
            } => {
                let result = client_manager
                    .is_client_connected(&client_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            ClientCommand::GetClientStatus {
                client_id,
                respond_to,
            } => {
                let result = client_manager
                    .get_client_status(&client_id)
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
            ClientCommand::GetAllClientStatuses { respond_to } => {
                let result = client_manager
                    .get_all_client_statuses()
                    .await
                    .map_err(|e| ActorError::DatabaseError(e.to_string()));
                let _ = respond_to.send(result);
            }
        }
    }

    /// Send not initialized error for job command
    fn send_not_initialized_error_job(cmd: JobCommand) {
        match cmd {
            JobCommand::CreateJob { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::ClaimNextJob { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::StartJob { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::UpdateProgress { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::UpdatePhase { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::CompleteJob { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::FailJob { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::RequeueJob { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::GetJob { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::GetActiveJobs { respond_to } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::GetPendingCount { respond_to } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::GetActiveCount { respond_to } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            JobCommand::GetMediaFileFrames { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
        }
    }

    /// Send not initialized error for client command
    fn send_not_initialized_error_client(cmd: ClientCommand) {
        match cmd {
            ClientCommand::RegisterClient { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            ClientCommand::UpdateHeartbeat { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            ClientCommand::DisconnectClient { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            ClientCommand::DisconnectAndRequeueJobs { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            ClientCommand::GetClient { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            ClientCommand::GetConnectedClients { respond_to } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            ClientCommand::IsClientConnected { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            ClientCommand::GetClientStatus { respond_to, .. } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
            ClientCommand::GetAllClientStatuses { respond_to } => {
                let _ = respond_to.send(Err(ActorError::NotInitialized));
            }
        }
    }

    /// Shutdown the actor and clean up resources
    async fn shutdown(&mut self) {
        info!("Shutting down job management actor");

        // Cancel background tasks
        if let Some(handle) = self.scheduler_handle.take() {
            debug!("Aborting job scheduler task");
            handle.abort();
        }
        if let Some(handle) = self.heartbeat_handle.take() {
            debug!("Aborting heartbeat monitor task");
            handle.abort();
        }

        // Close database pool
        if let Some(pool) = self.pool.take() {
            debug!("Closing database pool");
            pool.close().await;
        }

        info!("Job management actor shutdown complete");
    }
}
