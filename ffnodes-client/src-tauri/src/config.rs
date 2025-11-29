use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub server_url: String,
    pub server_guid: String,
    pub display_name: String,
    pub computer_name: String,
    pub machine_id: String,
    pub client_id: Option<String>,
    pub auth_token: Option<String>,
    pub auto_start_processing: Option<bool>,
}

impl ClientConfig {
    pub fn new(server_url: String, server_guid: String, display_name: String) -> Self {
        let computer_name = hostname::get()
            .ok()
            .and_then(|h| h.into_string().ok())
            .unwrap_or_else(|| "Unknown".to_string());

        // Generate machine ID from hardware fingerprint
        let machine_id = crate::machine_id::generate_machine_id()
            .unwrap_or_else(|e| {
                tracing::warn!("Failed to generate machine ID: {}", e);
                uuid::Uuid::new_v4().to_string()
            });

        Self {
            server_url,
            server_guid,
            display_name,
            computer_name,
            machine_id,
            client_id: None,
            auth_token: None,
            auto_start_processing: Some(false),
        }
    }

    fn config_path() -> PathBuf {
        let config_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("FFNodes");

        fs::create_dir_all(&config_dir).ok();
        config_dir.join("config.json")
    }

    pub fn load() -> anyhow::Result<Option<Self>> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;

        // Try to deserialize with machine_id
        let mut config: ClientConfig = match serde_json::from_str(&content) {
            Ok(cfg) => cfg,
            Err(_) => {
                // Migration: Old config without machine_id
                // Deserialize to a temporary struct that's missing machine_id
                #[derive(Deserialize)]
                struct OldConfig {
                    server_url: String,
                    server_guid: String,
                    display_name: String,
                    computer_name: String,
                    client_id: Option<String>,
                    auth_token: Option<String>,
                    auto_start_processing: Option<bool>,
                }

                let old_config: OldConfig = serde_json::from_str(&content)?;

                // Generate machine_id for old config
                let machine_id = crate::machine_id::generate_machine_id()
                    .unwrap_or_else(|e| {
                        tracing::warn!("Failed to generate machine ID during migration: {}", e);
                        uuid::Uuid::new_v4().to_string()
                    });

                tracing::info!("Migrated old config to include machine_id");

                // Create new config with machine_id
                ClientConfig {
                    server_url: old_config.server_url,
                    server_guid: old_config.server_guid,
                    display_name: old_config.display_name,
                    computer_name: old_config.computer_name,
                    machine_id,
                    client_id: old_config.client_id,
                    auth_token: old_config.auth_token,
                    auto_start_processing: old_config.auto_start_processing,
                }
            }
        };

        // Provide default for auto_start_processing if not present
        if config.auto_start_processing.is_none() {
            config.auto_start_processing = Some(false);
        }

        // Save migrated config if machine_id was just added
        config.save().ok();

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
