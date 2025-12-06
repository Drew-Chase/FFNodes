use crate::gpu::GpuInfo;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use ffmpeg::FFMpeg;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::Mutex;

#[derive(Debug, Clone, serde::Serialize)]
pub struct EncodingProgress {
    pub frame: i64,
    pub fps: f64,
    pub bitrate: f64,
    pub speed: f64,
    pub percentage: f64,
}

#[derive(Default)]
struct ProgressState {
    frame: i64,
    fps: f64,
    bitrate: f64,
    speed: f64,
}

pub struct Encoder {
    ffmpeg: FFMpeg,
    temp_dir: PathBuf,
    current_ffmpeg_pid: Arc<Mutex<Option<u32>>>,
}

impl Encoder {
    pub async fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("ffnodes-client");
        std::fs::create_dir_all(&temp_dir)?;

        // Use the FFmpeg library for auto-detection/download
        let mut ffmpeg = FFMpeg::default();
        ffmpeg.fetch().await?;

        log::info!("FFmpeg binary located at: {:?}", ffmpeg);

        Ok(Self {
            ffmpeg,
            temp_dir,
            current_ffmpeg_pid: Arc::new(Mutex::new(None)),
        })
    }

    /// Abort any currently running FFmpeg process
    pub async fn abort_encoding(&self) {
        let mut pid_lock = self.current_ffmpeg_pid.lock().await;
        if let Some(pid) = *pid_lock {
            log::info!("Killing FFmpeg process with PID: {}", pid);
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000) // CREATE_NO_WINDOW
                    .output();
            }
            #[cfg(not(target_os = "windows"))]
            {
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
            *pid_lock = None;
            log::info!("✓ FFmpeg process killed");
        }
    }

    /// Extract a random frame from the video for background
    pub async fn extract_frame(&self, video_path: &Path) -> Result<String> {
        log::info!("Extracting random frame from: {}", video_path.display());

        // Get video duration first
        let duration = self.get_video_duration(video_path).await?;
        log::debug!("Video duration: {:.2}s", duration);

        // Pick a random timestamp (avoid first and last 10%)
        let random_time = (duration * 0.1) + (duration * 0.8 * rand::random::<f64>());
        let percentage = (random_time / duration) * 100.0;
        log::info!(
            "Extracting frame at random position: {:.2}s ({:.1}% through video)",
            random_time,
            percentage
        );

        // Output path
        let output_path = self.temp_dir.join(format!(
            "frame_{}.jpg",
            video_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("video")
        ));
        log::debug!("Output path: {}", output_path.display());

        // Extract frame using FFmpeg builder
        let builder = self
            .ffmpeg
            .ffmpeg_command_builder()
            .input(video_path.to_str().unwrap())?
            .start_time(random_time.to_string())?
            .output(output_path.to_str().unwrap())?
            .raw_arg("-vframes".to_string())
            .raw_arg("1".to_string())
            .raw_arg("-q:v".to_string())
            .raw_arg("2".to_string())
            .overwrite(true);

        let cmd = builder.build()?;
        log::debug!("Extract frame FFmpeg command: {}", cmd);

        cmd.execute(None, None)
            .await
            .map_err(|e| anyhow!("Failed to extract frame: {}", e))?;

        log::info!("Frame extracted successfully");

        // Read the frame and convert to base64
        let frame_data = tokio::fs::read(&output_path).await?;
        let base64_frame = format!(
            "data:image/jpeg;base64,{}",
            general_purpose::STANDARD.encode(&frame_data)
        );

        // delete the frame file
        fs::remove_file(output_path).await?;

        Ok(base64_frame)
    }

    /// Get video duration in seconds
    async fn get_video_duration(&self, video_path: &Path) -> Result<f64> {
        log::debug!("Getting duration for: {}", video_path.display());

        use ffmpeg::builders::common::ProbeFormat;
        let probe = self
            .ffmpeg
            .ffprobe_command_builder()
            .input(video_path.to_str().unwrap())?
            .show_format()
            .output_format(ProbeFormat::JSON)
            .log_level(16) // error level
            .hide_banner();

        let cmd = probe.build()?;
        let result = cmd
            .execute_json()
            .await
            .map_err(|e| anyhow!("Failed to probe video: {}", e))?;

        let duration_str = result
            .format
            .and_then(|f| f.duration)
            .ok_or_else(|| anyhow!("No duration found in probe result"))?;

        let duration: f64 = duration_str
            .parse()
            .map_err(|e| anyhow!("Failed to parse duration: {}", e))?;

        log::debug!("Duration: {:.2}s", duration);
        Ok(duration)
    }

    /// Encode video with progress callback
    pub async fn encode_video<F>(
        &self,
        input_path: &Path,
        output_path: &Path,
        gpu_info: &GpuInfo,
        ffmpeg_template: &str,
        total_frames: Option<i64>,
        mut progress_callback: F,
    ) -> Result<(i64, i64, f64)>
    where
        F: FnMut(EncodingProgress) + Send,
    {
        log::info!(
            "Starting encoding: {} -> {}",
            input_path.display(),
            output_path.display()
        );
        log::debug!("FFmpeg template: {}", ffmpeg_template);

        // Parse template and build FFmpeg command
        let mut args =
            self.build_ffmpeg_args(input_path, output_path, gpu_info, ffmpeg_template)?;
        log::debug!("FFmpeg args before progress: {:?}", args);

        // Insert -progress pipe:2 before the last argument (output file)
        // FFmpeg requires -progress to come before the output file
        if !args.is_empty() {
            let output_file = args
                .pop()
                .ok_or_else(|| anyhow!("No output file in args"))?;
            args.push("-progress".to_string());
            args.push("pipe:2".to_string());
            args.push(output_file);
        }
        log::debug!("FFmpeg args after progress: {:?}", args);

        // Get total frames for percentage calculation
        // Use server-provided frames if available, otherwise probe the file
        let total_frames = match total_frames {
            Some(frames) => {
                log::info!("Using server-provided frame count: {}", frames);
                frames
            }
            None => {
                log::debug!("No frame count from server, probing file...");
                let frames = self.get_total_frames(input_path).await?;
                log::info!("Probed frame count: {}", frames);
                frames
            }
        };

        // Create a channel to receive FFmpeg output
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        // Build and execute FFmpeg command using the builder
        // Note: args already contain input (-i), progress, and output from template substitution
        let mut builder = self.ffmpeg.ffmpeg_command_builder();
        builder = builder.overwrite(true);

        // Add all parsed args as raw arguments (includes input, progress, and output)
        builder = builder.raw_args(args);

        // Skip validation since input/output are in raw args, not registered via .input()/.output()
        builder = builder.skip_validation();

        let cmd = builder.build()?;
        log::info!("Starting FFmpeg encoding process");
        log::debug!("FFmpeg command: {}", cmd);

        // Execute in background and process progress with PID storage
        let pid_storage = self.current_ffmpeg_pid.clone();
        let handle =
            tokio::spawn(async move { cmd.execute_with_pid(None, Some(tx), pid_storage).await });

        // Process progress updates with stateful accumulation
        let mut progress_state = ProgressState::default();
        let mut speed_samples = Vec::new();
        log::debug!("Starting progress monitoring loop");
        while let Some(line) = rx.recv().await {
            log::trace!("Encoder received line: {}", line.trim());
            if let Some(progress) =
                self.parse_progress_update(&line, &mut progress_state, total_frames)
            {
                log::trace!(
                    "Progress: frame={}, fps={:.1}, speed={:.2}x, {}%",
                    progress.frame,
                    progress.fps,
                    progress.speed,
                    progress.percentage
                );
                // Track speed for average calculation
                if progress.speed > 0.0 {
                    speed_samples.push(progress.speed);
                }
                progress_callback(progress);
            }
        }
        log::debug!("Progress monitoring loop ended");

        // Wait for encoding to complete
        handle
            .await?
            .map_err(|e| anyhow!("Encoding failed: {}", e))?;

        log::info!("Encoding completed successfully");

        // Get output file stats
        let metadata = tokio::fs::metadata(output_path).await?;
        let output_size = metadata.len() as i64;
        log::debug!("Output file size: {} bytes", output_size);

        // Calculate output bitrate (approximation)
        let duration = self.get_video_duration(input_path).await?;
        let output_bitrate = ((output_size * 8) as f64 / duration) as i64;
        log::debug!("Output bitrate: {} bps", output_bitrate);

        // Calculate average speed
        let average_speed = if !speed_samples.is_empty() {
            speed_samples.iter().sum::<f64>() / speed_samples.len() as f64
        } else {
            0.0
        };
        log::debug!(
            "Average encoding speed: {:.2}x (from {} samples)",
            average_speed,
            speed_samples.len()
        );

        Ok((output_size, output_bitrate, average_speed))
    }

    /// Build FFmpeg arguments from template
    fn build_ffmpeg_args(
        &self,
        input_path: &Path,
        output_path: &Path,
        gpu_info: &GpuInfo,
        template: &str,
    ) -> Result<Vec<String>> {
        log::debug!("Building FFmpeg command from template");
        log::debug!("Template: {}", template);
        log::debug!("Input: {}", input_path.display());
        log::debug!("Output: {}", output_path.display());
        log::debug!("GPU encoder: {}", gpu_info.encoder_h264);

        // Replace template variables with quoted paths
        let command = template
            .replace("{INPUT}", &format!("\"{}\"", input_path.display()))
            .replace("{OUTPUT}", &format!("\"{}\"", output_path.display()))
            .replace(
                "{HWACCEL_CODE}",
                &format!("_{}", gpu_info.encoder_h264.replace("h264_", "")),
            );

        log::debug!("Template after substitution: {}", command);

        // Use proper shell-like parsing instead of split_whitespace
        let args = shlex::split(&command)
            .ok_or_else(|| anyhow!("Failed to parse FFmpeg command template"))?;

        log::info!("FFmpeg command built successfully with {} args", args.len());
        log::debug!("Full command: {:?}", args);

        // Note: -progress pipe:2 is added in encode_video(), not here
        Ok(args)
    }

    /// Get total frames in video
    async fn get_total_frames(&self, video_path: &Path) -> Result<i64> {
        log::debug!("Getting total frames for: {}", video_path.display());

        use ffmpeg::builders::common::{ProbeFormat, StreamType};
        let probe = self
            .ffmpeg
            .ffprobe_command_builder()
            .input(video_path.to_str().unwrap())?
            .select_streams(StreamType::Video)
            .count_packets()
            .show_streams()
            .output_format(ProbeFormat::JSON)
            .log_level(16) // error level
            .hide_banner();

        let cmd = probe.build()?;
        let result = cmd
            .execute_json()
            .await
            .map_err(|e| anyhow!("Failed to probe video: {}", e))?;

        let frames_str = result
            .streams
            .first()
            .and_then(|s| s.nb_read_packets.clone())
            .ok_or_else(|| anyhow!("No packet count found in probe result"))?;

        let frames: i64 = frames_str
            .parse()
            .map_err(|e| anyhow!("Failed to parse frame count: {}", e))?;

        log::debug!("Total frames: {}", frames);
        Ok(frames)
    }

    /// Parse FFmpeg progress update with stateful accumulation
    /// FFmpeg with `-progress pipe:2` sends each field on a separate line:
    /// frame=123
    /// fps=45.6
    /// bitrate=1234.5kbits/s
    /// speed=1.2x
    /// progress=continue
    fn parse_progress_update(
        &self,
        line: &str,
        state: &mut ProgressState,
        total_frames: i64,
    ) -> Option<EncodingProgress> {
        let line = line.trim();

        // Parse key=value format
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim();
            match key {
                "frame" => {
                    // Frame value can be in two formats:
                    // 1. Structured: "frame=12345" (just the number)
                    // 2. Visual status line: "frame= 2957 fps=410 q=-0.0 size=..." (number + extra text)
                    // Extract just the number part
                    let frame_str = value.split_whitespace().next().unwrap_or("0");
                    let parsed = frame_str.parse().unwrap_or(0);
                    log::trace!("Parsed frame: {} (from '{}')", parsed, value);
                    state.frame = parsed;
                }
                "fps" => {
                    let parsed = value.parse().unwrap_or(0.0);
                    log::trace!("Parsed fps: {} (from '{}')", parsed, value);
                    state.fps = parsed;
                }
                "bitrate" => {
                    // Handle format: "1234.5kbits/s"
                    let bitrate_str = value.replace("kbits/s", "").trim().to_string();
                    let parsed = bitrate_str.parse().unwrap_or(0.0) * 1000.0;
                    log::trace!("Parsed bitrate: {} (from '{}')", parsed, value);
                    state.bitrate = parsed;
                }
                "speed" => {
                    // Handle format: "1.2x"
                    let speed_str = value.replace("x", "").trim().to_string();
                    let parsed = speed_str.parse().unwrap_or(0.0);
                    log::trace!("Parsed speed: {} (from '{}')", parsed, value);
                    state.speed = parsed;
                }
                "progress" => {
                    log::trace!("Progress marker: '{}', state.frame={}", value, state.frame);
                    // When we see "progress=continue" or "progress=end", emit current state
                    if (value == "continue" || value == "end") && state.frame > 0 {
                        let percentage = if total_frames > 0 {
                            (state.frame as f64 / total_frames as f64) * 100.0
                        } else {
                            0.0
                        };

                        log::debug!(
                            "Emitting progress: frame={}, fps={:.1}, speed={:.2}x, {:.1}%",
                            state.frame,
                            state.fps,
                            state.speed,
                            percentage
                        );

                        return Some(EncodingProgress {
                            frame: state.frame,
                            fps: state.fps,
                            bitrate: state.bitrate,
                            speed: state.speed,
                            percentage,
                        });
                    }
                }
                _ => {} // Ignore unknown keys
            }
        }

        None
    }
}
