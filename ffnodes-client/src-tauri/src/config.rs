use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
    pub server_guid: String,
    pub display_name: String,
    pub computer_name: String,
    pub client_id: Option<String>,
    pub auth_token: Option<String>,
}

impl ClientConfig {
    pub fn new(server_url: String, server_guid: String, display_name: String) -> Self {
        let computer_name = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "Unknown".to_string());

        Self {
            server_url,
            server_guid,
            display_name,
            computer_name,
            client_id: None,
            auth_token: None,
        }
    }

    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ffnodes-client");

        fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.json")
    }

    pub fn load() -> anyhow::Result<Option<Self>> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let config: ClientConfig = serde_json::from_str(&content)?;
        Ok(Some(config))
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn delete() -> anyhow::Result<()> {
        let path = Self::config_path();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
