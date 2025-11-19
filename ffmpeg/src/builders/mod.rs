//! FFmpeg and FFprobe command builders
//!
//! This module provides fluent, type-safe builders for constructing FFmpeg and FFprobe commands.
//!
//! # Examples
//!
//! ## Basic video conversion with FFmpeg
//! ```no_run
//! use ffmpeg::builders::FFmpegBuilder;
//! use ffmpeg::builders::common::{VideoCodec, AudioCodec, Preset};
//! use ffmpeg::FFMpeg;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ffmpeg = FFMpeg::default();
//! ffmpeg.fetch().await?;
//! let ffmpeg = Arc::new(ffmpeg);
//!
//! let cmd = FFmpegBuilder::new(ffmpeg)
//!     .input("input.mp4")?
//!     .output("output.mp4")?
//!         .video_codec(VideoCodec::H264)
//!         .crf(23)
//!         .preset(Preset::Fast)
//!         .audio_codec(AudioCodec::AAC)
//!         .audio_bitrate("192k")?
//!     .overwrite(true)
//!     .build()?;
//!
//! cmd.execute(None, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Probing media metadata with FFprobe
//! ```no_run
//! use ffmpeg::builders::FFprobeBuilder;
//! use ffmpeg::builders::common::ProbeFormat;
//! use ffmpeg::FFMpeg;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut ffmpeg = FFMpeg::default();
//! ffmpeg.fetch().await?;
//! let ffmpeg = Arc::new(ffmpeg);
//!
//! let cmd = FFprobeBuilder::new(ffmpeg)
//!     .input("video.mp4")?
//!     .show_streams()
//!     .show_format()
//!     .output_format(ProbeFormat::JSON)
//!     .build()?;
//!
//! let metadata = cmd.execute_json().await?;
//! if let Some((width, height)) = metadata.video_resolution() {
//!     println!("Resolution: {}x{}", width, height);
//! }
//! if let Some(duration) = metadata.duration_seconds() {
//!     println!("Duration: {:.2} seconds", duration);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Hardware-accelerated encoding
//! ```no_run
//! use ffmpeg::builders::FFmpegBuilder;
//! use ffmpeg::builders::common::{VideoCodec, HardwareAccel, Preset};
//! use ffmpeg::FFMpeg;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ffmpeg = Arc::new(FFMpeg::default());
//!
//! let cmd = FFmpegBuilder::new(ffmpeg)
//!     .hardware_accel(HardwareAccel::CUDA)
//!     .input("input.mp4")?
//!     .output("output.mp4")?
//!         .video_codec_hw(VideoCodec::H264, HardwareAccel::NVENC)
//!         .preset(Preset::Fast)
//!     .overwrite(true)
//!     .build()?;
//!
//! cmd.execute(None, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Complex filtering
//! ```no_run
//! use ffmpeg::builders::FFmpegBuilder;
//! use ffmpeg::builders::common::VideoCodec;
//! use ffmpeg::FFMpeg;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ffmpeg = Arc::new(FFMpeg::default());
//!
//! let cmd = FFmpegBuilder::new(ffmpeg)
//!     .input("input.mp4")?
//!     .output("output.mp4")?
//!         .video_codec(VideoCodec::H264)
//!         .width(1920)  // Scale to 1920 width, maintaining aspect ratio
//!         .video_filter("fps=30")  // Set framerate to 30fps
//!         .audio_filter("volume=2.0")  // Double the volume
//!     .overwrite(true)
//!     .build()?;
//!
//! cmd.execute(None, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Two-pass encoding
//! ```no_run
//! use ffmpeg::builders::FFmpegBuilder;
//! use ffmpeg::builders::common::VideoCodec;
//! use ffmpeg::FFMpeg;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ffmpeg = Arc::new(FFMpeg::default());
//!
//! // First pass
//! let cmd1 = FFmpegBuilder::new(ffmpeg.clone())
//!     .input("input.mp4")?
//!     .output("/dev/null")?
//!         .video_codec(VideoCodec::H264)
//!         .video_bitrate("2M")?
//!         .two_pass_first("passlog")
//!     .overwrite(true)
//!     .build()?;
//! cmd1.execute(None, None).await?;
//!
//! // Second pass
//! let cmd2 = FFmpegBuilder::new(ffmpeg)
//!     .input("input.mp4")?
//!     .output("output.mp4")?
//!         .video_codec(VideoCodec::H264)
//!         .video_bitrate("2M")?
//!         .two_pass_second("passlog")
//!     .overwrite(true)
//!     .build()?;
//! cmd2.execute(None, None).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## HLS streaming output
//! ```no_run
//! use ffmpeg::builders::FFmpegBuilder;
//! use ffmpeg::builders::common::VideoCodec;
//! use ffmpeg::FFMpeg;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let ffmpeg = Arc::new(FFMpeg::default());
//!
//! let cmd = FFmpegBuilder::new(ffmpeg)
//!     .input("input.mp4")?
//!     .output("output.m3u8")?
//!         .video_codec(VideoCodec::H264)
//!         .hls_options(6, 5, "segment_%03d.ts")
//!     .overwrite(true)
//!     .build()?;
//!
//! cmd.execute(None, None).await?;
//! # Ok(())
//! # }
//! ```

pub mod common;
pub mod error;
pub mod ffmpeg_builder;
pub mod ffprobe_builder;

pub use common::*;
pub use error::{BuilderError, Result};
pub use ffmpeg_builder::{FFmpegBuilder, FFmpegCommand};
pub use ffprobe_builder::{
    Chapter, FFprobeBuilder, FFprobeCommand, Format, ProbeResult, Program, Stream,
};
