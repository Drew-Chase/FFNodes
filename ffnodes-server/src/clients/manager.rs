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

    /// Register a new client
    pub async fn register_client(
        &self,
        display_name: String,
        computer_name: String,
    ) -> Result<Client> {
        let client = Client::new(Uuid::new_v4().to_string(), display_name, computer_name);

        sqlx::query(
            r#"INSERT INTO clients
            (id, display_name, computer_name, connected_at, last_heartbeat)
            VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(&client.id)
        .bind(&client.display_name)
        .bind(&client.computer_name)
        .bind(client.connected_at)
        .bind(client.last_heartbeat)
        .execute(&self.pool)
        .await?;

        // Add to in-memory registry
        let mut clients = self.clients.write().await;
        clients.push(client.clone());

        Ok(client)
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
    #[allow(dead_code)]
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
