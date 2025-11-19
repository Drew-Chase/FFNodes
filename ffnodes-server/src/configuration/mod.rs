use anyhow::Result;
use serde::{Deserialize, Serialize};
use ffmpeg::FFMpeg;

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub port: u16,
    #[serde(flatten)]
    pub ffmpeg: FFMpeg,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            port: 8080,
            ffmpeg: FFMpeg::default(),
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
        Ok(serde_json::from_reader(config_file.into_std().await)?)
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
