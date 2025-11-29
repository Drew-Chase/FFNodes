use crate::configuration::Configuration;
use anyhow::{anyhow, Result};
use ffmpeg::builders::{ContainerFormat, HardwareAccel, ProbeFormat, VideoCodec};
use log::{debug, error, info};
use serde_hash::HashIds;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub mod media_file_db;
pub mod progress;
mod scanner;
mod watcher;
pub use media_file_db::initialize;
pub use scanner::Scanner;
pub use watcher::FileWatcher;

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
    pub duration: f32,
    /// The width and height of the file, if it's available.
    pub width: u64,
    /// The width and height of the file, if it's available.
    pub height: u64,
    /// The number of frames in the file, if it's available.
    pub frames: u64,
    /// The last modified timestamp of the file
    pub last_modified: u64,
    /// Encoding complexity score (resolution × bitrate × duration)
    pub encoding_complexity: u64,
    /// Number of times encoding has failed for this file
    pub retry_count: u32,
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

    pub async fn from_path_with_config(
        file_path: impl AsRef<Path>,
        config: &Configuration,
    ) -> Result<Self> {
        info!("Probing media file {:?}", file_path.as_ref());

        // Try fast probe first, then retry with slow frame counting if needed
            let builder = config
                .ffmpeg
                .ffprobe_command_builder()
                .input(file_path.as_ref().to_string_lossy().to_string())?
                .show_streams()
                .log_level(1)
                .output_format(ProbeFormat::JSON)
                .show_format();

            let probe = builder
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

            let format = probe.format.as_ref().ok_or_else(|| {
                error!("Failed to parse ffprobe output - missing format data");
                anyhow!("Failed to parse ffprobe output")
            })?;
            let (width, height) = probe.video_resolution().ok_or_else(|| {
                error!("Failed to parse ffprobe output - missing video resolution");
                anyhow!("Failed to parse ffprobe output")
            })?;

            // Try to extract frame count from various sources
            let frames_result = Self::extract_frame_count(&probe).await?;

            let frames = match frames_result {
                Some(frames) => frames,
                None => {
                        // Even with slow counting, couldn't get frame count
                        return Err(anyhow!("Failed to determine frame count"));
                }
            };

            let duration: f32 = {
                if let Some(duration) = format.duration.as_ref() {
                    if let Ok(duration) = duration.parse::<f32>() {
                        duration
                    } else {
                        return Err(anyhow!("Failed to parse duration"));
                    }
                } else {
                    0.0f32
                }
            };

            let last_modified = tokio::fs::metadata(&file_path)
                .await?
                .modified()?
                .duration_since(UNIX_EPOCH)?
                .as_secs();

            let scanned_bit_rate = format
                .bit_rate
                .as_ref()
                .map(|bit_rate| bit_rate.parse().unwrap_or(0))
                .unwrap_or(0);

            // Calculate encoding complexity: (width × height) × bitrate × duration
            let encoding_complexity = ((width as u64 * height as u64) as f64
                * scanned_bit_rate as f64
                * duration as f64) as u64;

            debug!("Parsed ffprobe output successfully");

            Ok(Self {
                path: file_path.as_ref().to_path_buf(),
                scanned_size: format
                    .size
                    .as_ref()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0),
                size: None,
                scanned_bit_rate,
                bit_rate: None,
                duration,
                width: width as u64,
                height: height as u64,
                frames,
                last_modified,
                encoding_complexity,
                retry_count: 0,
                processed: false,
            })
    }

    /// Extract frame count from probe output, trying multiple sources
    async fn extract_frame_count(probe: &ffmpeg::builders::ProbeResult) -> Result<Option<u64>> {
        let stream = probe
            .first_video_stream()
            .ok_or_else(|| anyhow!("No video stream found"))?;

        // Try nb_frames field
        if let Some(nb_frames_s) = stream.nb_frames.as_ref()
            && let Ok(nb_frames) = nb_frames_s.parse::<u64>()
        {
            return Ok(Some(nb_frames));
        }

        // Try NUMBER_OF_FRAMES tag
        if let Some(nb_frames_s) = stream.tags.get("NUMBER_OF_FRAMES")
            && let Ok(nb_frames) = nb_frames_s.parse::<u64>()
        {
            return Ok(Some(nb_frames));
        }

        // Try NUMBER_OF_FRAMES-eng tag
        if let Some(nb_frames_s) = stream.tags.get("NUMBER_OF_FRAMES-eng")
            && let Ok(nb_frames) = nb_frames_s.parse::<u64>()
        {
            return Ok(Some(nb_frames));
        }

        // If unable to get the frame count from the probe output, use ffmpeg to try to get the frame count.
        let config = Configuration::load().await?;
        let cmd = config
            .ffmpeg
            .ffmpeg_command_builder()
            .hardware_accel(HardwareAccel::Auto)
            .video_codec(VideoCodec::Copy)
            .format(ContainerFormat::NULL)
            .input(
                probe
                    .clone()
                    .format
                    .ok_or_else(|| anyhow!("Failed to get the probe format"))?
                    .filename,
            )?
            .output("-")?
            .build()?;
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<String>(100);

        // Execute in background while we drain the receiver
        let execute_task = tokio::spawn(async move {
            cmd.execute(None, Some(sender)).await
        });

        // Drain receiver concurrently with execution
        let mut frame_count = None;
        while let Some(output) = receiver.recv().await {
            if output.contains("frame=")
                && let Some(section) = output.split("frame=").last()
                && let Some(count_str) = section.split_whitespace().next()
                && let Ok(count) = count_str.trim().parse::<u64>()
            {
                frame_count = Some(count);
            }
        }

        // Wait for execution to complete
        execute_task.await??;

        Ok(frame_count)
    }
}
