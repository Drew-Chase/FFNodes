use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Client registration payload
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientRegistration {
    pub server_guid: String,
    pub display_name: String,
    pub computer_name: String,
}

/// Client information stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Client {
    pub id: String,
    pub display_name: String,
    pub computer_name: String,
    pub connected_at: i64,
    pub last_heartbeat: i64,
    pub disconnected_at: Option<i64>,
}

impl Client {
    pub fn new(id: String, display_name: String, computer_name: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id,
            display_name,
            computer_name,
            connected_at: now,
            last_heartbeat: now,
            disconnected_at: None,
        }
    }

    pub fn is_connected(&self) -> bool {
        self.disconnected_at.is_none()
    }
}

/// Authentication response
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub auth_token: String,
    pub ffmpeg_template: String,
}

/// Heartbeat request
#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub client_id: String,
}

/// Client status for monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientStatus {
    pub id: String,
    pub display_name: String,
    pub computer_name: String,
    pub connected_at: i64,
    pub last_heartbeat: i64,
    pub active_jobs_count: usize,
}
