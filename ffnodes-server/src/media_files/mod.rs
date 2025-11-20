use crate::configuration::Configuration;
use anyhow::{Result, anyhow};
use ffmpeg::builders::ProbeFormat;
use serde_hash::HashIds;
use std::path::{Path, PathBuf};
use log::{debug, error, info};

mod media_file_db;
mod scanner;
pub use media_file_db::initialize;
pub use scanner::Scanner;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow, HashIds)]
/// A video media file.
pub struct MediaFile {
    /// The path to the file on disk.
    pub path: PathBuf,
    /// The original file size at the point of scanning.
    pub scanned_size: u64,
    /// The current file size, if the file has been processed.
    pub size: Option<u64>,
    /// The bit rate of the file at the point of scanning, if available.
    pub scanned_bit_rate: u64,
    /// The bit rate of the file, if it has been processed.
    pub bit_rate: Option<u64>,
    /// The duration of the file, if it's available.
    pub duration: u64,
    /// The width and height of the file, if it's available.
    pub width: u64,
    /// The width and height of the file, if it's available.
    pub height: u64,
    pub frames: u64,
    /// Whether the file has been processed.
    pub processed: bool,
}

impl MediaFile {
    #[allow(dead_code)]
    pub async fn new(file_path: impl AsRef<Path>) -> Result<Self> {
        info!("Loading media file {:?}", file_path.as_ref());
        let config = Configuration::load().await?;
        Self::from_path_with_config(file_path, &config).await
    }

    pub async fn from_path_with_config(file_path: impl AsRef<Path>, config: &Configuration) -> Result<Self> {
        info!("Probing media file {:?}", file_path.as_ref());
        let probe = config
            .ffmpeg
            .ffprobe_command_builder()
            .input(file_path.as_ref().to_string_lossy().to_string())?
            .show_streams()
            .log_level(1)
            .output_format(ProbeFormat::JSON)
            .show_format()
            .build()
            .map_err(|e| {
                error!("Failed to build ffprobe command: {}", e);
                e
            })?
            .execute_json()
            .await
            .map_err(|e| {
                error!("Failed to execute ffprobe command: {}", e);
                e
            })?;

        let format = probe
            .format
            .as_ref()
            .ok_or_else(|| {
                error!("Failed to parse ffprobe output - missing format data");
                anyhow!("Failed to parse ffprobe output")
            })?;
        let (width, height) = probe
            .video_resolution()
            .ok_or_else(|| {
                error!("Failed to parse ffprobe output - missing video resolution");
                anyhow!("Failed to parse ffprobe output")
            })?;
        let frames = probe
            .first_video_stream()
            .ok_or_else(|| {
                error!("Failed to parse ffprobe output - missing video stream");
                anyhow!("Failed to parse ffprobe output")
            })?
            .nb_frames
            .as_ref()
            .map(|s| s.parse().unwrap_or(0))
            .unwrap_or(0);

        debug!("Parsed ffprobe output successfully");

        Ok(Self {
            path: file_path.as_ref().to_path_buf(),
            scanned_size: format
                .size
                .as_ref()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            size: None,
            scanned_bit_rate: format
                .bit_rate
                .as_ref()
                .map(|bit_rate| bit_rate.parse().unwrap_or(0))
                .unwrap_or(0),
            bit_rate: None,
            duration: format
                .duration
                .as_ref()
                .map(|duration| duration.parse().unwrap_or(0))
                .unwrap_or(0),
            width: width as u64,
            height: height as u64,
            frames: frames as u64,
            processed: false,
        })
    }
}
