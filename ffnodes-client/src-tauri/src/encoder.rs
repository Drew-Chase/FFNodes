use crate::gpu::GpuInfo;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose, Engine as _};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, serde::Serialize)]
pub struct EncodingProgress {
    pub frame: i64,
    pub fps: f64,
    pub bitrate: f64,
    pub speed: f64,
    pub percentage: f64,
}

pub struct Encoder {
    ffmpeg_path: PathBuf,
    temp_dir: PathBuf,
}

impl Encoder {
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("ffnodes-client");
        std::fs::create_dir_all(&temp_dir)?;

        // Try to find FFmpeg from the library
        let ffmpeg_path = which::which("ffmpeg")
            .or_else(|_| {
                // Try common locations
                if cfg!(windows) {
                    which::which("C:\\ffmpeg\\bin\\ffmpeg.exe")
                } else {
                    which::which("/usr/local/bin/ffmpeg")
                }
            })
            .unwrap_or_else(|_| PathBuf::from("ffmpeg"));

        Ok(Self {
            ffmpeg_path,
            temp_dir,
        })
    }

    /// Extract a random frame from the video for background
    pub async fn extract_frame(&self, video_path: &Path) -> Result<String> {
        // Get video duration first
        let duration = self.get_video_duration(video_path).await?;

        // Pick a random timestamp (avoid first and last 10%)
        let random_time = (duration * 0.1) + (duration * 0.8 * rand::random::<f64>());

        // Output path
        let output_path = self.temp_dir.join(format!(
            "frame_{}.jpg",
            video_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("video")
        ));

        // Extract frame using FFmpeg
        let output = Command::new(&self.ffmpeg_path)
            .args([
                "-ss",
                &random_time.to_string(),
                "-i",
                video_path.to_str().unwrap(),
                "-vframes",
                "1",
                "-q:v",
                "2",
                "-y",
                output_path.to_str().unwrap(),
            ])
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow!("Failed to extract frame: {:?}", output.stderr));
        }

        // Read the frame and convert to base64
        let frame_data = tokio::fs::read(&output_path).await?;
        let base64_frame = format!(
            "data:image/jpeg;base64,{}",
            general_purpose::STANDARD.encode(&frame_data)
        );

        Ok(base64_frame)
    }

    /// Get video duration in seconds
    async fn get_video_duration(&self, video_path: &Path) -> Result<f64> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "default=noprint_wrappers=1:nokey=1",
                video_path.to_str().unwrap(),
            ])
            .output()
            .await?;

        let duration_str = String::from_utf8(output.stdout)?;
        let duration: f64 = duration_str.trim().parse()?;

        Ok(duration)
    }

    /// Encode video with progress callback
    pub async fn encode_video<F>(
        &self,
        input_path: &Path,
        output_path: &Path,
        gpu_info: &GpuInfo,
        ffmpeg_template: &str,
        mut progress_callback: F,
    ) -> Result<(i64, i64)>
    where
        F: FnMut(EncodingProgress) + Send,
    {
        // Parse template and build FFmpeg command
        let args = self.build_ffmpeg_args(input_path, output_path, gpu_info, ffmpeg_template)?;

        log::info!("Starting encoding: {:?}", args);

        // Get total frames for percentage calculation
        let total_frames = self.get_total_frames(input_path).await?;

        // Start FFmpeg process
        let mut child = Command::new(&self.ffmpeg_path)
            .args(&args)
            .stderr(Stdio::piped())
            .spawn()?;

        // Read stderr for progress
        let stderr = child.stderr.take().ok_or_else(|| anyhow!("Failed to capture stderr"))?;
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if let Some(progress) = self.parse_progress_line(&line, total_frames) {
                progress_callback(progress);
            }
        }

        // Wait for completion
        let status = child.wait().await?;

        if !status.success() {
            return Err(anyhow!("Encoding failed with status: {:?}", status));
        }

        // Get output file stats
        let metadata = tokio::fs::metadata(output_path).await?;
        let output_size = metadata.len() as i64;

        // Calculate output bitrate (approximation)
        let duration = self.get_video_duration(input_path).await?;
        let output_bitrate = ((output_size * 8) as f64 / duration) as i64;

        Ok((output_size, output_bitrate))
    }

    /// Build FFmpeg arguments from template
    fn build_ffmpeg_args(
        &self,
        input_path: &Path,
        output_path: &Path,
        gpu_info: &GpuInfo,
        template: &str,
    ) -> Result<Vec<String>> {
        let mut args = Vec::new();

        // Replace template variables
        let command = template
            .replace("{INPUT}", input_path.to_str().unwrap())
            .replace("{OUTPUT}", output_path.to_str().unwrap())
            .replace("{HWACCEL_CODE}", &format!("_{}", gpu_info.encoder_h264.replace("h264_", "")));

        // Parse command string into args
        // Simple space-based splitting (in production, use proper shell parsing)
        for arg in command.split_whitespace() {
            args.push(arg.to_string());
        }

        // Add progress reporting
        args.push("-progress".to_string());
        args.push("pipe:2".to_string());

        Ok(args)
    }

    /// Get total frames in video
    async fn get_total_frames(&self, video_path: &Path) -> Result<i64> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-count_packets",
                "-show_entries",
                "stream=nb_read_packets",
                "-of",
                "csv=p=0",
                video_path.to_str().unwrap(),
            ])
            .output()
            .await?;

        let frames_str = String::from_utf8(output.stdout)?;
        let frames: i64 = frames_str.trim().parse().unwrap_or(0);

        Ok(frames)
    }

    /// Parse FFmpeg progress line
    fn parse_progress_line(&self, line: &str, total_frames: i64) -> Option<EncodingProgress> {
        // FFmpeg progress format: "frame=123 fps=45.6 ... bitrate=1234.5kbits/s speed=1.2x"
        let mut frame = 0i64;
        let mut fps = 0.0;
        let mut bitrate = 0.0;
        let mut speed = 0.0;

        for part in line.split_whitespace() {
            if let Some(value) = part.strip_prefix("frame=") {
                frame = value.parse().unwrap_or(0);
            } else if let Some(value) = part.strip_prefix("fps=") {
                fps = value.parse().unwrap_or(0.0);
            } else if let Some(value) = part.strip_prefix("bitrate=") {
                let bitrate_str = value.replace("kbits/s", "");
                bitrate = bitrate_str.parse().unwrap_or(0.0) * 1000.0; // Convert to bits/s
            } else if let Some(value) = part.strip_prefix("speed=") {
                let speed_str = value.replace("x", "");
                speed = speed_str.parse().unwrap_or(0.0);
            }
        }

        if frame > 0 {
            let percentage = if total_frames > 0 {
                (frame as f64 / total_frames as f64) * 100.0
            } else {
                0.0
            };

            Some(EncodingProgress {
                frame,
                fps,
                bitrate,
                speed,
                percentage,
            })
        } else {
            None
        }
    }
}
