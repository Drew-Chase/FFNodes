use super::common::{ProbeFormat, StreamType};
use super::error::{BuilderError, Result};
use crate::FFMpeg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use encoding_rs::UTF_16LE;

/// FFprobe command builder with fluent API
///
/// # Example
/// ```no_run
/// use ffmpeg::builders::FFprobeBuilder;
/// use ffmpeg::builders::common::ProbeFormat;
/// use ffmpeg::FFMpeg;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ffmpeg = Arc::new(FFMpeg::default());
/// let probe = FFprobeBuilder::new(ffmpeg)
///     .input("video.mp4")?
///     .show_streams()
///     .show_format()
///     .output_format(ProbeFormat::JSON);
///
/// let cmd = probe.build()?;
/// let metadata = cmd.execute_json().await?;
/// println!("Duration: {:?}", metadata.format.and_then(|f| f.duration));
/// # Ok(())
/// # }
/// ```
pub struct FFprobeBuilder {
    ffmpeg: Arc<FFMpeg>,
    input: Option<String>,
    args: Vec<String>,
    output_format: ProbeFormat,
}

impl FFprobeBuilder {
    /// Create a new FFprobe command builder
    pub fn new(ffmpeg: Arc<FFMpeg>) -> Self {
        Self {
            ffmpeg,
            input: None,
            args: Vec::new(),
            output_format: ProbeFormat::JSON,
        }
    }

    /// Set the input file to probe
    pub fn input(mut self, path: impl Into<String>) -> Result<Self> {
        self.input = Some(path.into());
        Ok(self)
    }

    /// Set the output format
    pub fn output_format(mut self, format: ProbeFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Show stream information
    pub fn show_streams(mut self) -> Self {
        self.args.push("-show_streams".to_string());
        self
    }

    /// Show format information
    pub fn show_format(mut self) -> Self {
        self.args.push("-show_format".to_string());
        self
    }

    /// Show chapter information
    pub fn show_chapters(mut self) -> Self {
        self.args.push("-show_chapters".to_string());
        self
    }

    /// Show program information
    pub fn show_programs(mut self) -> Self {
        self.args.push("-show_programs".to_string());
        self
    }

    /// Show packet information
    pub fn show_packets(mut self) -> Self {
        self.args.push("-show_packets".to_string());
        self
    }

    /// Show frame information
    pub fn show_frames(mut self) -> Self {
        self.args.push("-show_frames".to_string());
        self
    }

    /// Select specific streams by type
    pub fn select_streams(mut self, stream_type: StreamType) -> Self {
        self.args.push("-select_streams".to_string());
        self.args.push(stream_type.to_string());
        self
    }

    /// Count frames
    pub fn count_frames(mut self) -> Self {
        self.args.push("-count_frames".to_string());
        self
    }

    /// Count packets
    pub fn count_packets(mut self) -> Self {
        self.args.push("-count_packets".to_string());
        self
    }

    /// Show data in binary format
    pub fn show_data(mut self) -> Self {
        self.args.push("-show_data".to_string());
        self
    }

    /// Show data hash
    pub fn show_data_hash(mut self, algorithm: impl Into<String>) -> Self {
        self.args.push("-show_data_hash".to_string());
        self.args.push(algorithm.into());
        self
    }

    /// Show private data
    pub fn show_private_data(mut self) -> Self {
        self.args.push("-show_private_data".to_string());
        self
    }

    /// Show pixel format
    pub fn show_pixel_format(mut self) -> Self {
        self.args.push("-show_pixel_formats".to_string());
        self
    }

    /// Set log level
    pub fn log_level(mut self, level: impl Into<u8>) -> Self {
        self.args.push("-loglevel".to_string());
        self.args.push(level.into().to_string());
        self
    }

    /// Hide banner
    pub fn hide_banner(mut self) -> Self {
        self.args.push("-hide_banner".to_string());
        self
    }

    /// Pretty print output
    pub fn pretty(mut self) -> Self {
        self.args.push("-pretty".to_string());
        self
    }

    /// Add a raw argument (escape hatch for unsupported options)
    pub fn raw_arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple raw arguments
    pub fn raw_args(mut self, args: impl IntoIterator<Item = String>) -> Self {
        self.args.extend(args);
        self
    }

    /// Build the command and return an executable command
    pub fn build(self) -> Result<FFprobeCommand> {
        if self.input.is_none() {
            return Err(BuilderError::NoInput);
        }

        let mut args = Vec::new();

        // Output format
        args.push("-print_format".to_string());
        args.push(self.output_format.to_string());

        // Other arguments
        args.extend(self.args);

        // Input file
        args.push("-i".to_string());
        args.push(self.input.unwrap());

        Ok(FFprobeCommand {
            ffmpeg: self.ffmpeg,
            args,
            output_format: self.output_format,
        })
    }
}

/// Executable FFprobe command
pub struct FFprobeCommand {
    ffmpeg: Arc<FFMpeg>,
    args: Vec<String>,
    output_format: ProbeFormat,
}

impl FFprobeCommand {
    /// Get the command arguments as a vector of strings
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Execute the command and return raw output
    pub async fn execute(&self, working_dir: Option<PathBuf>) -> Result<String> {
        let working_dir = working_dir.unwrap_or_else(|| PathBuf::from("."));
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);

        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        self.ffmpeg
            .exec_ffprobe(&args, working_dir, tx)
            .await
            .map_err(|e| BuilderError::ExecutionError(e.to_string()))?;

        // Collect all output chunks as bytes first
        let mut raw_bytes = Vec::new();
        while let Some(chunk) = rx.recv().await {
            raw_bytes.extend_from_slice(chunk.as_bytes());
        }

        // Detect and handle encoding
        let output = if raw_bytes.len() >= 2 && raw_bytes[0] == 0xFF && raw_bytes[1] == 0xFE {
            // UTF-16 LE with BOM
            let (decoded, _encoding, had_errors) = UTF_16LE.decode(&raw_bytes[2..]); // Skip BOM
            if had_errors {
                eprintln!("Warning: UTF-16 LE decoding had errors");
            }
            decoded.into_owned()
        } else if raw_bytes.len() >= 2 && raw_bytes[0] == 0xFE && raw_bytes[1] == 0xFF {
            // UTF-16 BE with BOM
            let (decoded, _encoding, had_errors) = encoding_rs::UTF_16BE.decode(&raw_bytes[2..]);
            if had_errors {
                eprintln!("Warning: UTF-16 BE decoding had errors");
            }
            decoded.into_owned()
        } else if raw_bytes.len() >= 4 && raw_bytes[0] == b'{' && raw_bytes[1] == 0 && raw_bytes[2] == b' ' && raw_bytes[3] == 0 {
            // UTF-16 LE without BOM (detected by pattern: '{' 0x00 ' ' 0x00 for "{ ")
            let (decoded, _encoding, had_errors) = UTF_16LE.decode(&raw_bytes);
            if had_errors {
                eprintln!("Warning: UTF-16 LE (no BOM) decoding had errors");
            }
            decoded.into_owned()
        } else {
            // Try UTF-8
            match String::from_utf8(raw_bytes.clone()) {
                Ok(s) => s,
                Err(_) => {
                    // UTF-8 failed, try UTF-16 LE as last resort
                    let (decoded, _encoding, _) = UTF_16LE.decode(&raw_bytes);
                    decoded.into_owned()
                }
            }
        };

        Ok(output)
    }

    /// Execute the command and parse JSON output
    pub async fn execute_json(&self) -> Result<ProbeResult> {
        if self.output_format != ProbeFormat::JSON {
            return Err(BuilderError::InvalidOption(
                "Output format must be JSON to use execute_json()".to_string(),
            ));
        }

        let output = self.execute(None).await?;

        let result: ProbeResult = serde_json::from_str(&output)
            .map_err(|e| {
                eprintln!("=== FFprobe JSON Parse Error ===");
                eprintln!("Error: {}", e);
                eprintln!("Output length: {} bytes", output.len());
                eprintln!("First 500 chars: {}",
                    if output.len() > 500 { &output[..500] } else { &output });
                if output.len() > 500 {
                    eprintln!("Last 100 chars: {}", &output[output.len().saturating_sub(100)..]);
                }
                BuilderError::ParseError(format!("Failed to parse JSON: {}", e))
            })?;

        Ok(result)
    }
}

impl std::fmt::Display for FFprobeCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ffprobe {}", self.args.join(" "))
    }
}

// ==================== JSON Structures ====================

/// FFprobe result containing streams, format, and other metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeResult {
    #[serde(default)]
    pub streams: Vec<Stream>,
    #[serde(default)]
    pub format: Option<Format>,
    #[serde(default)]
    pub chapters: Vec<Chapter>,
    #[serde(default)]
    pub programs: Vec<Program>,
}

/// Stream information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub index: u32,
    pub codec_name: Option<String>,
    pub codec_long_name: Option<String>,
    pub profile: Option<String>,
    pub codec_type: Option<String>,
    pub codec_time_base: Option<String>,
    pub codec_tag_string: Option<String>,
    pub codec_tag: Option<String>,

    // Video-specific
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub coded_width: Option<u32>,
    pub coded_height: Option<u32>,
    pub has_b_frames: Option<u32>,
    pub sample_aspect_ratio: Option<String>,
    pub display_aspect_ratio: Option<String>,
    pub pix_fmt: Option<String>,
    pub level: Option<i32>,
    pub color_range: Option<String>,
    pub color_space: Option<String>,
    pub color_transfer: Option<String>,
    pub color_primaries: Option<String>,
    pub field_order: serde_json::Value,
    pub refs: Option<u32>,

    // Audio-specific
    pub sample_fmt: Option<String>,
    pub sample_rate: Option<String>,
    pub channels: Option<u32>,
    pub channel_layout: Option<String>,
    pub bits_per_sample: Option<u32>,

    // Common
    pub r_frame_rate: Option<String>,
    pub avg_frame_rate: Option<String>,
    pub time_base: Option<String>,
    pub start_pts: serde_json::Value,
    pub start_time: Option<String>,
    pub duration_ts: Option<i64>,
    pub duration: Option<String>,
    pub bit_rate: Option<String>,
    pub max_bit_rate: Option<String>,
    pub bits_per_raw_sample: Option<String>,
    pub nb_frames: Option<String>,
    pub nb_read_frames: Option<String>,
    pub nb_read_packets: Option<String>,

    // Tags
    #[serde(default)]
    pub tags: HashMap<String, String>,

    // Disposition
    #[serde(default)]
    pub disposition: serde_json::Value,
}

/// Format/container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Format {
    pub filename: String,
    pub nb_streams: u32,
    pub nb_programs: Option<u32>,
    pub format_name: String,
    pub format_long_name: Option<String>,
    pub start_time: Option<String>,
    pub duration: Option<String>,
    pub size: Option<String>,
    pub bit_rate: Option<String>,
    pub probe_score: Option<u32>,

    #[serde(default)]
    pub tags: HashMap<String, String>,
}

/// Chapter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: i64,
    pub time_base: String,
    pub start: i64,
    pub start_time: String,
    pub end: i64,
    pub end_time: String,

    #[serde(default)]
    pub tags: HashMap<String, String>,
}

/// Program information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub program_id: u32,
    pub program_num: u32,
    pub nb_streams: u32,
    pub pmt_pid: Option<u32>,
    pub pcr_pid: Option<u32>,
    pub start_pts: Option<i64>,
    pub start_time: Option<String>,
    pub end_pts: Option<i64>,
    pub end_time: Option<String>,

    #[serde(default)]
    pub tags: HashMap<String, String>,

    #[serde(default)]
    pub streams: Vec<Stream>,
}

impl ProbeResult {
    /// Get the first video stream
    pub fn first_video_stream(&self) -> Option<&Stream> {
        self.streams
            .iter()
            .find(|s| s.codec_type.as_deref() == Some("video"))
    }

    /// Get the first audio stream
    pub fn first_audio_stream(&self) -> Option<&Stream> {
        self.streams
            .iter()
            .find(|s| s.codec_type.as_deref() == Some("audio"))
    }

    /// Get all video streams
    pub fn video_streams(&self) -> Vec<&Stream> {
        self.streams
            .iter()
            .filter(|s| s.codec_type.as_deref() == Some("video"))
            .collect()
    }

    /// Get all audio streams
    pub fn audio_streams(&self) -> Vec<&Stream> {
        self.streams
            .iter()
            .filter(|s| s.codec_type.as_deref() == Some("audio"))
            .collect()
    }

    /// Get all subtitle streams
    pub fn subtitle_streams(&self) -> Vec<&Stream> {
        self.streams
            .iter()
            .filter(|s| s.codec_type.as_deref() == Some("subtitle"))
            .collect()
    }

    /// Get duration as f64 seconds (from format or first stream)
    pub fn duration_seconds(&self) -> Option<f64> {
        self.format
            .as_ref()
            .and_then(|f| f.duration.as_ref())
            .and_then(|d| d.parse::<f64>().ok())
    }

    /// Get bit rate as u64
    pub fn bit_rate(&self) -> Option<u64> {
        self.format
            .as_ref()
            .and_then(|f| f.bit_rate.as_ref())
            .and_then(|b| b.parse::<u64>().ok())
    }

    /// Get video resolution (width, height) from first video stream
    pub fn video_resolution(&self) -> Option<(u32, u32)> {
        self.first_video_stream()
            .and_then(|s| s.width.zip(s.height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_builder_basic() {
        let ffmpeg = Arc::new(FFMpeg::default());
        let probe = FFprobeBuilder::new(ffmpeg)
            .input("video.mp4")
            .unwrap()
            .show_streams()
            .show_format()
            .output_format(ProbeFormat::JSON);

        let cmd = probe.build().unwrap();
        let args = cmd.args();

        assert!(args.contains(&"-print_format".to_string()));
        assert!(args.contains(&"json".to_string()));
        assert!(args.contains(&"-show_streams".to_string()));
        assert!(args.contains(&"-show_format".to_string()));
        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"video.mp4".to_string()));
    }

    #[test]
    fn test_probe_builder_no_input_error() {
        let ffmpeg = Arc::new(FFMpeg::default());
        let probe = FFprobeBuilder::new(ffmpeg).show_streams();

        let result = probe.build();
        assert!(matches!(result, Err(BuilderError::NoInput)));
    }
}
