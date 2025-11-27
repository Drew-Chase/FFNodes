use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use ffmpeg::FFMpeg;
use uuid::Uuid;

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub port: u16,
    #[serde(flatten)]
    pub ffmpeg: FFMpeg,
    pub watch_directories: Vec<PathBuf>,
    pub server_guid: String,
    pub jwt_secret: String,
    pub ffmpeg_template: String,
    pub client_timeout_seconds: u64,
    pub notify_batch_interval_seconds: u64,
    pub max_concurrent_jobs_per_client: u32,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            port: 8080,
            ffmpeg: FFMpeg::default(),
            watch_directories: vec![],
            server_guid: Uuid::new_v4().to_string(),
            jwt_secret: Uuid::new_v4().to_string(),
            ffmpeg_template: "-i {INPUT} -c:v h264{HWACCEL_CODE} -c:a aac {OUTPUT}".to_string(),
            client_timeout_seconds: 300,
            notify_batch_interval_seconds: 30,
            max_concurrent_jobs_per_client: 4,
        }
    }
}

impl Configuration {
    pub async fn load() -> Result<Self> {
        if !tokio::fs::try_exists(CONFIG_FILE_NAME).await? {
            let mut default_config = Self::default();
            default_config.reset().await?;
            return Ok(default_config);
        }
        let config_file = tokio::fs::File::open(CONFIG_FILE_NAME).await?;
        let mut config: Configuration = serde_json::from_reader(config_file.into_std().await)?;

        // Ensure jwt_secret exists (for backwards compatibility)
        if config.jwt_secret.is_empty() {
            config.jwt_secret = Uuid::new_v4().to_string();
            config.save().await?;
        }

        // Canonicalize watch directories for proper path comparison
        config.watch_directories = config.watch_directories
            .into_iter()
            .filter_map(|p| p.canonicalize().ok())
            .collect();

        Ok(config)
    }
    pub async fn save(&self) -> Result<()> {
        let config_file = tokio::fs::File::create(CONFIG_FILE_NAME).await?;
        serde_json::to_writer_pretty(config_file.into_std().await, self)?;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<()> {
        *self = Self::default();
	    self.ffmpeg.fetch().await?;
        self.save().await?;
        Ok(())
    }
}
