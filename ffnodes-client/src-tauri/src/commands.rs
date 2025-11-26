use crate::api::{HandshakeRequest, ServerClient};
use crate::config::ClientConfig;
use crate::gpu::{detect_gpu, GpuInfo};
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

    let request = HandshakeRequest {
        server_guid: input.server_guid.clone(),
        display_name: input.display_name.clone(),
        computer_name: hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "Unknown".to_string()),
    };

    let response = client
        .handshake(request)
        .await
        .map_err(|e| format!("Handshake failed: {}", e))?;

    // Generate client ID
    let client_id = uuid::Uuid::new_v4().to_string();

    let mut config = ClientConfig::new(input.server_url, input.server_guid, input.display_name);
    config.client_id = Some(client_id);
    config.auth_token = Some(response.auth_token);
    config.ffmpeg_template = Some(response.ffmpeg_template);

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

    encoder
        .extract_frame(path)
        .await
        .map_err(|e| e.to_string())
}

/// Get list of active jobs from server
#[tauri::command]
pub async fn get_active_jobs(config: ClientConfig) -> Result<Vec<crate::api::EncodingJob>, String> {
    let client = ServerClient::new(config.server_url);
    client
        .get_active_jobs()
        .await
        .map_err(|e| e.to_string())
}

/// Start job processing
#[tauri::command]
pub async fn start_job_processing(
    job_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>>,
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
    job_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>>,
) -> Result<(), String> {
    let manager = job_manager.lock().await;
    manager.stop().await.map_err(|e| e.to_string())
}

/// Pause job processing
#[tauri::command]
pub async fn pause_job_processing(
    job_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>>,
) -> Result<(), String> {
    let manager = job_manager.lock().await;
    manager.pause().await.map_err(|e| e.to_string())
}

/// Resume job processing
#[tauri::command]
pub async fn resume_job_processing(
    job_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>>,
) -> Result<(), String> {
    let manager = job_manager.lock().await;
    manager.resume().await.map_err(|e| e.to_string())
}

/// Get job manager state
#[tauri::command]
pub async fn get_job_manager_state(
    job_manager: tauri::State<'_, std::sync::Arc<tokio::sync::Mutex<crate::job_manager::JobManager>>>,
) -> Result<crate::job_manager::JobManagerState, String> {
    let manager = job_manager.lock().await;
    Ok(manager.get_state().await)
}

/// Log a message from the frontend
#[tauri::command]
pub fn log_frontend(level: String, message: String, source: Option<String>) {
    crate::logger::log_frontend(level, message, source);
}

/// Get the FFmpeg command with template variables filled in
#[tauri::command]
pub async fn get_ffmpeg_command(config: ClientConfig, gpu: GpuInfo) -> Result<String, String> {
    let template = config.ffmpeg_template
        .ok_or_else(|| "No FFmpeg template configured".to_string())?;

    // Fill in template variables with example values
    let command = template
        .replace("{INPUT}", "input.mp4")
        .replace("{OUTPUT}", "output.mp4")
        .replace("{HWACCEL_CODE}", &format!("_{}", gpu.encoder_h264.replace("h264_", "")));

    Ok(command)
}
