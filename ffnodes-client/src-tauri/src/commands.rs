use crate::api::{HandshakeRequest, ServerClient};
use crate::config::ClientConfig;
use crate::gpu::{GpuInfo, detect_gpu};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigInput {
    pub server_url: String,
    pub server_guid: String,
    pub display_name: String,
}

/// Load saved configuration
#[tauri::command]
pub async fn load_config() -> Result<Option<ClientConfig>, String> {
    ClientConfig::load().map_err(|e| e.to_string())
}

/// Save configuration
#[tauri::command]
pub async fn save_config(config: ClientConfig) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())
}

/// Test server connection and perform handshake
#[tauri::command]
pub async fn test_connection(input: ConfigInput) -> Result<ClientConfig, String> {
    let client = ServerClient::new(input.server_url.clone());

    // Create config first to generate machine_id
    let mut config = ClientConfig::new(input.server_url, input.server_guid, input.display_name);

    let request = HandshakeRequest {
        server_guid: config.server_guid.clone(),
        display_name: config.display_name.clone(),
        computer_name: config.computer_name.clone(),
        machine_id: config.machine_id.clone(),
    };

    let response = client
        .handshake(request)
        .await
        .map_err(|e| format!("Handshake failed: {}", e))?;

    config.client_id = Some(response.client_id);
    config.auth_token = Some(response.auth_token);
    // Note: ffmpeg_template is NOT saved to config - server sends it with each job

    config.save().map_err(|e| e.to_string())?;

    Ok(config)
}

/// Detect GPU and return encoder information
#[tauri::command]
pub async fn get_gpu_info() -> Result<GpuInfo, String> {
    detect_gpu().map_err(|e| e.to_string())
}

/// Extract a random frame from video for background
#[tauri::command]
pub async fn extract_frame(video_path: String) -> Result<String, String> {
    use crate::encoder::Encoder;
    use std::path::Path;

    let encoder = Encoder::new().await.map_err(|e| e.to_string())?;
    let path = Path::new(&video_path);

    encoder.extract_frame(path).await.map_err(|e| e.to_string())
}

/// Get list of active jobs from server
#[tauri::command]
pub async fn get_active_jobs(config: ClientConfig) -> Result<Vec<crate::api::EncodingJob>, String> {
    let auth_token = config.auth_token.ok_or("No auth token")?;
    let client = ServerClient::with_auth(config.server_url, auth_token);
    client.get_active_jobs().await.map_err(|e| e.to_string())
}

/// Start job processing
#[tauri::command]
pub async fn start_job_processing(
    job_manager: tauri::State<
        '_,
        std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>,
    >,
    config: ClientConfig,
    gpu: GpuInfo,
) -> Result<(), String> {
    let manager = job_manager.lock().await;
    manager.set_config(config).await;
    manager.set_gpu_info(gpu).await;
    manager.start().await.map_err(|e| e.to_string())
}

/// Stop job processing
#[tauri::command]
pub async fn stop_job_processing(
    job_manager: tauri::State<
        '_,
        std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>,
    >,
) -> Result<(), String> {
    let manager = job_manager.lock().await;
    manager.stop().await.map_err(|e| e.to_string())
}

/// Pause job processing
#[tauri::command]
pub async fn pause_job_processing(
    job_manager: tauri::State<
        '_,
        std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>,
    >,
) -> Result<(), String> {
    let manager = job_manager.lock().await;
    manager.pause().await.map_err(|e| e.to_string())
}

/// Resume job processing
#[tauri::command]
pub async fn resume_job_processing(
    job_manager: tauri::State<
        '_,
        std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>,
    >,
) -> Result<(), String> {
    let manager = job_manager.lock().await;
    manager.resume().await.map_err(|e| e.to_string())
}

/// Get job manager state
#[tauri::command]
pub async fn get_job_manager_state(
    job_manager: tauri::State<
        '_,
        std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>,
    >,
) -> Result<crate::job_manager::JobManagerState, String> {
    let manager = job_manager.lock().await;
    Ok(manager.get_state().await)
}

/// Log a message from the frontend
#[tauri::command]
pub fn log_frontend(level: String, message: String, source: Option<String>) {
    crate::logger::log_frontend(level, message, source);
}

/// Get client history statistics
#[tauri::command]
pub async fn get_client_history(config: ClientConfig) -> Result<crate::api::ClientHistoryResponse, String> {
    let auth_token = config.auth_token.ok_or("No auth token")?;
    let client_id = config.client_id.ok_or("No client ID")?;
    let client = ServerClient::with_auth(config.server_url, auth_token);
    client.get_client_history(&client_id).await.map_err(|e| e.to_string())
}

/// Get remote user progress
#[tauri::command]
pub async fn get_remote_progress(config: ClientConfig) -> Result<crate::api::RemoteProgressResponse, String> {
    let auth_token = config.auth_token.ok_or("No auth token")?;
    let client_id = config.client_id.clone();
    let client = ServerClient::with_auth(config.server_url, auth_token);
    client.get_remote_progress(client_id.as_deref()).await.map_err(|e| e.to_string())
}

/// Get leaderboard data
#[tauri::command]
pub async fn get_leaderboard(config: ClientConfig, category: String) -> Result<crate::api::LeaderboardResponse, String> {
    let auth_token = config.auth_token.ok_or("No auth token")?;
    let client = ServerClient::with_auth(config.server_url, auth_token);
    client.get_leaderboard(&category).await.map_err(|e| e.to_string())
}
