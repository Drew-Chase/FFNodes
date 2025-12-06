use std::path::PathBuf;
use anyhow::{anyhow, Result};
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
    pub output_container: String,
    pub client_timeout_seconds: u64,
    pub notify_batch_interval_seconds: u64,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            port: 7456,
            ffmpeg: FFMpeg::default(),
            watch_directories: vec![],
            server_guid: Uuid::new_v4().to_string(),
            jwt_secret: Uuid::new_v4().to_string(),
            ffmpeg_template: r#"-i {INPUT} -map 0 -c:v h264{HWACCEL_CODE} -b:v 5M -maxrate 8M -bufsize 8M -profile:v high -vf "scale='min(1920,iw)':-2" -c:a aac -b:a 320k {OUTPUT}"#.to_string(),
            output_container: "mkv".to_string(),
            client_timeout_seconds: 300,
            notify_batch_interval_seconds: 30,
        }
    }
}

impl Configuration {
    pub async fn load() -> Result<Self> {
        if !tokio::fs::try_exists(CONFIG_FILE_NAME).await? {
            let mut default_config = Self::default();
            default_config.reset().await?;
            default_config.parse_env()?;
            default_config.save().await?;
            return Ok(default_config);
        }
        let config_file = tokio::fs::File::open(CONFIG_FILE_NAME).await?;
        let mut config: Configuration = serde_json::from_reader(config_file.into_std().await)?;

        // Ensure jwt_secret exists (for backwards compatibility)
        if config.jwt_secret.is_empty() {
            config.jwt_secret = Uuid::new_v4().to_string();
            config.save().await?;
        }

        // Ensure output_container exists (for backwards compatibility)
        if config.output_container.is_empty() {
            config.output_container = "mp4".to_string();
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

    fn parse_env(&mut self) -> Result<()> {
        if let Ok(s_port) = std::env::var("FFNODE_PORT") {
            self.port = s_port.parse().map_err(|_| anyhow!("FFNODE_PORT must be a valid integer"))?;
        }
        if let Ok(container) = std::env::var("FFNODE_OUTPUT_CONTAINER") {
            self.output_container = container;
        }

        if let Ok(directories) = std::env::var("FFNODE_WATCH_DIRECTORIES") {
            self.watch_directories = serde_json::from_str(directories.as_str())?;
        }

        if let Ok(ffmpeg_template) = std::env::var("FFNODE_FFMPEG_TEMPLATE") {
            self.ffmpeg_template = ffmpeg_template;
        }

        Ok(())
    }

    pub async fn reset(&mut self) -> Result<()> {
        *self = Self::default();
	    self.ffmpeg.fetch().await?;
        self.save().await?;
        Ok(())
    }
}
