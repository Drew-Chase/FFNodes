use super::models::{Client, ClientStatus};
use anyhow::{anyhow, Result};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Client manager
pub struct ClientManager {
    pool: SqlitePool,
    // In-memory registry for fast lookups
    clients: Arc<RwLock<Vec<Client>>>,
}

impl ClientManager {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Find existing client by machine_id or computer_name
    /// Returns the existing client if found, None otherwise
    async fn find_existing_client(
        &self,
        computer_name: &str,
        machine_id: Option<&str>,
    ) -> Result<Option<Client>> {
        // Strategy 1: Try to find by machine_id first (most reliable)
        if let Some(mid) = machine_id {
            let client: Option<Client> = sqlx::query_as(
                "SELECT * FROM clients WHERE machine_id = ? ORDER BY last_heartbeat DESC LIMIT 1"
            )
            .bind(mid)
            .fetch_optional(&self.pool)
            .await?;

            if client.is_some() {
                tracing::debug!("Found existing client by machine_id");
                return Ok(client);
            }
        }

        // Strategy 2: Fallback to computer_name for old clients without machine_id
        let client: Option<Client> = sqlx::query_as(
            "SELECT * FROM clients WHERE computer_name = ? ORDER BY last_heartbeat DESC LIMIT 1"
        )
        .bind(computer_name)
        .fetch_optional(&self.pool)
        .await?;

        if client.is_some() {
            tracing::debug!("Found existing client by computer_name");
        }

        Ok(client)
    }

    /// Register a new client or reconnect existing one
    /// Implements find-or-create pattern with hardware-based machine identification
    pub async fn register_client(
        &self,
        display_name: String,
        computer_name: String,
        machine_id: Option<String>,
    ) -> Result<Client> {
        // Try to find existing client
        let existing = self
            .find_existing_client(&computer_name, machine_id.as_deref())
            .await?;

        if let Some(mut client) = existing {
            // RECONNECTION PATH - Update existing client
            let now = chrono::Utc::now().timestamp();

            // Build UPDATE query for changed fields
            let mut updates = vec!["last_heartbeat = ?", "disconnected_at = NULL"];
            let display_name_changed = client.display_name != display_name;
            let computer_name_changed = client.computer_name != computer_name;
            let machine_id_added = client.machine_id.is_none() && machine_id.is_some();

            if display_name_changed {
                updates.push("display_name = ?");
            }
            if computer_name_changed {
                updates.push("computer_name = ?");
            }
            if machine_id_added {
                updates.push("machine_id = ?");
            }

            let query = format!(
                "UPDATE clients SET {} WHERE id = ?",
                updates.join(", ")
            );

            // Build query with dynamic parameters
            let mut q = sqlx::query(&query).bind(now);
            if display_name_changed {
                q = q.bind(&display_name);
            }
            if computer_name_changed {
                q = q.bind(&computer_name);
            }
            if machine_id_added {
                q = q.bind(&machine_id);
            }
            q = q.bind(&client.id);

            q.execute(&self.pool).await?;

            // Update struct for return
            client.display_name = display_name;
            client.computer_name = computer_name;
            if machine_id_added {
                client.machine_id = machine_id;
            }
            client.last_heartbeat = now;
            client.disconnected_at = None;

            // Update in-memory registry
            let mut clients = self.clients.write().await;
            if let Some(c) = clients.iter_mut().find(|c| c.id == client.id) {
                *c = client.clone();
            } else {
                clients.push(client.clone());
            }

            tracing::info!("Client reconnected: {} ({})", client.display_name, client.id);
            Ok(client)
        } else {
            // NEW CLIENT PATH - Create new record
            let client = Client::new(
                Uuid::new_v4().to_string(),
                display_name,
                computer_name,
                machine_id,
            );

            sqlx::query(
                r#"INSERT INTO clients
                (id, display_name, computer_name, machine_id, connected_at, last_heartbeat)
                VALUES (?, ?, ?, ?, ?, ?)"#,
            )
            .bind(&client.id)
            .bind(&client.display_name)
            .bind(&client.computer_name)
            .bind(&client.machine_id)
            .bind(client.connected_at)
            .bind(client.last_heartbeat)
            .execute(&self.pool)
            .await?;

            // Add to in-memory registry
            let mut clients = self.clients.write().await;
            clients.push(client.clone());

            tracing::info!("New client registered: {} ({})", client.display_name, client.id);
            Ok(client)
        }
    }

    /// Update client heartbeat
    pub async fn update_heartbeat(&self, client_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        let result = sqlx::query(
            r#"UPDATE clients
            SET last_heartbeat = ?
            WHERE id = ? AND disconnected_at IS NULL"#,
        )
        .bind(now)
        .bind(client_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(anyhow!("Client not found or already disconnected"));
        }

        // Update in-memory registry
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.iter_mut().find(|c| c.id == client_id) {
            client.last_heartbeat = now;
        }

        Ok(())
    }

    /// Disconnect a client
    pub async fn disconnect_client(&self, client_id: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"UPDATE clients
            SET disconnected_at = ?
            WHERE id = ?"#,
        )
        .bind(now)
        .bind(client_id)
        .execute(&self.pool)
        .await?;

        // Remove from in-memory registry
        let mut clients = self.clients.write().await;
        clients.retain(|c| c.id != client_id);

        tracing::info!("Client disconnected: {}", client_id);

        Ok(())
    }

    /// Disconnect a client and requeue their jobs
    pub async fn disconnect_and_requeue_jobs(
        &self,
        client_id: &str,
        job_queue: &crate::jobs::JobQueue,
    ) -> Result<()> {
        tracing::info!("Disconnecting client {} and requeuing jobs", client_id);

        // Get all jobs assigned to this client
        let jobs = job_queue.get_client_jobs(client_id).await?;
        let job_count = jobs.len();

        // Requeue each job
        for job in jobs {
            if let Err(e) = job_queue.requeue_job(&job.id).await {
                tracing::error!("Failed to requeue job {}: {}", job.id, e);
            } else {
                tracing::info!("Requeued job {} from client {}", job.id, client_id);
            }
        }

        // Disconnect the client
        self.disconnect_client(client_id).await?;

        tracing::info!(
            "Client {} disconnected, {} jobs requeued",
            client_id,
            job_count
        );

        Ok(())
    }

    /// Get client by ID
    pub async fn get_client(&self, client_id: &str) -> Result<Option<Client>> {
        let client: Option<Client> = sqlx::query_as(r#"SELECT * FROM clients WHERE id = ?"#)
            .bind(client_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(client)
    }

    /// Get all connected clients
    pub async fn get_connected_clients(&self) -> Result<Vec<Client>> {
        let clients: Vec<Client> = sqlx::query_as(
            r#"SELECT * FROM clients
            WHERE disconnected_at IS NULL
            ORDER BY last_heartbeat DESC"#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(clients)
    }

    /// Get client status with active jobs count
    pub async fn get_client_status(&self, client_id: &str) -> Result<Option<ClientStatus>> {
        let client = self.get_client(client_id).await?;

        if let Some(client) = client {
            // Get active jobs count for this client
            let count: (i64,) = sqlx::query_as(
                r#"SELECT COUNT(*) FROM encoding_jobs
                WHERE assigned_client = ? AND status IN ('assigned', 'in_progress')"#,
            )
            .bind(client_id)
            .fetch_one(&self.pool)
            .await?;

            Ok(Some(ClientStatus {
                id: client.id,
                display_name: client.display_name,
                computer_name: client.computer_name,
                connected_at: client.connected_at,
                last_heartbeat: client.last_heartbeat,
                active_jobs_count: count.0 as usize,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all client statuses
    pub async fn get_all_client_statuses(&self) -> Result<Vec<ClientStatus>> {
        let clients = self.get_connected_clients().await?;
        let mut statuses = Vec::new();

        for client in clients {
            if let Some(status) = self.get_client_status(&client.id).await? {
                statuses.push(status);
            }
        }

        Ok(statuses)
    }

    /// Check if a client is connected
    pub async fn is_client_connected(&self, client_id: &str) -> Result<bool> {
        let client = self.get_client(client_id).await?;
        Ok(client.map(|c| c.is_connected()).unwrap_or(false))
    }

    /// Refresh in-memory client registry from database
    #[allow(dead_code)]
    pub async fn refresh_registry(&self) -> Result<()> {
        let connected_clients = self.get_connected_clients().await?;
        let mut clients = self.clients.write().await;
        *clients = connected_clients;
        Ok(())
    }
}
