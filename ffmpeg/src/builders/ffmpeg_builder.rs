use super::common::*;
use super::error::{BuilderError, Result};
use crate::FFMpeg;
use std::path::PathBuf;
use std::sync::Arc;

/// FFmpeg command builder with fluent API
///
/// # Example
/// ```no_run
/// use ffmpeg::builders::FFmpegBuilder;
/// use ffmpeg::builders::common::{VideoCodec, AudioCodec, Preset};
/// use ffmpeg::FFMpeg;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ffmpeg = Arc::new(FFMpeg::default());
/// let builder = FFmpegBuilder::new(ffmpeg)
///     .input("input.mp4")?
///         .start_time("00:01:30")?
///         .duration("00:05:00")?
///     .output("output.mp4")?
///         .video_codec(VideoCodec::H264)
///         .crf(23)
///         .preset(Preset::Fast)
///         .audio_codec(AudioCodec::AAC)
///         .audio_bitrate("192k")?
///     .overwrite(true);
///
/// let cmd = builder.build()?;
/// cmd.execute(None, None).await?;
/// # Ok(())
/// # }
/// ```
pub struct FFmpegBuilder {
    ffmpeg: Arc<FFMpeg>,
    pre_input_args: Vec<String>,
    inputs: Vec<InputSpec>,
    post_input_args: Vec<String>,
    outputs: Vec<OutputSpec>,
    global_args: Vec<String>,
    current_input: Option<InputSpec>,
    current_output: Option<OutputSpec>,
}

#[derive(Debug, Clone)]
struct InputSpec {
    path: String,
    options: Vec<String>,
}

#[derive(Debug, Clone)]
struct OutputSpec {
    path: String,
    options: Vec<String>,
}

impl FFmpegBuilder {
    /// Create a new FFmpeg command builder
    pub fn new(ffmpeg: Arc<FFMpeg>) -> Self {
        Self {
            ffmpeg,
            pre_input_args: Vec::new(),
            inputs: Vec::new(),
            post_input_args: Vec::new(),
            outputs: Vec::new(),
            global_args: Vec::new(),
            current_input: None,
            current_output: None,
        }
    }

    // ==================== Global Options ====================

    /// Set overwrite mode for output files
    pub fn overwrite(mut self, yes: bool) -> Self {
        if yes {
            self.global_args.push("-y".to_string());
        } else {
            self.global_args.push("-n".to_string());
        }
        self
    }

    /// Set hardware acceleration (applies globally before inputs)
    pub fn hardware_accel(mut self, accel: HardwareAccel) -> Self {
        if accel != HardwareAccel::None {
            self.pre_input_args.push("-hwaccel".to_string());
            self.pre_input_args.push(accel.to_string());
        }
        self
    }

    /// Set hardware device
    pub fn hardware_device(mut self, device: impl Into<String>) -> Self {
        self.pre_input_args.push("-hwaccel_device".to_string());
        self.pre_input_args.push(device.into());
        self
    }

    /// Set log level
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.global_args.push("-loglevel".to_string());
        self.global_args.push(level.into());
        self
    }

    /// Hide banner
    pub fn hide_banner(mut self) -> Self {
        self.global_args.push("-hide_banner".to_string());
        self
    }

    /// Add a raw argument (escape hatch for unsupported options)
    pub fn raw_arg(mut self, arg: impl Into<String>) -> Self {
        self.global_args.push(arg.into());
        self
    }

    /// Add multiple raw arguments
    pub fn raw_args(mut self, args: impl IntoIterator<Item = String>) -> Self {
        self.global_args.extend(args);
        self
    }

    // ==================== Input Handling ====================

    /// Add an input file
    pub fn input(mut self, path: impl Into<String>) -> Result<Self> {
        // Finalize any pending input
        if let Some(input) = self.current_input.take() {
            self.inputs.push(input);
        }

        self.current_input = Some(InputSpec {
            path: path.into(),
            options: Vec::new(),
        });
        Ok(self)
    }

    /// Set the input format
    pub fn input_format(mut self, format: impl Into<String>) -> Self {
        if let Some(input) = &mut self.current_input {
            input.options.push("-f".to_string());
            input.options.push(format.into());
        }
        self
    }

    /// Set start time for input (seeking)
    pub fn start_time(mut self, time: impl Into<String>) -> Result<Self> {
        let time = time.into();
        if let Some(input) = &mut self.current_input {
            input.options.push("-ss".to_string());
            input.options.push(time);
        }
        Ok(self)
    }

    /// Set duration for input
    pub fn duration(mut self, duration: impl Into<String>) -> Result<Self> {
        let duration = duration.into();
        if let Some(input) = &mut self.current_input {
            input.options.push("-t".to_string());
            input.options.push(duration);
        }
        Ok(self)
    }

    /// Set end time for input
    pub fn end_time(mut self, time: impl Into<String>) -> Result<Self> {
        let time = time.into();
        if let Some(input) = &mut self.current_input {
            input.options.push("-to".to_string());
            input.options.push(time);
        }
        Ok(self)
    }

    /// Set input framerate
    pub fn input_framerate(mut self, fps: impl Into<String>) -> Self {
        if let Some(input) = &mut self.current_input {
            input.options.push("-r".to_string());
            input.options.push(fps.into());
        }
        self
    }

    // ==================== Output Handling ====================

    /// Add an output file and finalize current input if any
    pub fn output(mut self, path: impl Into<String>) -> Result<Self> {
        // Finalize any pending input
        if let Some(input) = self.current_input.take() {
            self.inputs.push(input);
        }

        // Finalize any pending output
        if let Some(output) = self.current_output.take() {
            self.outputs.push(output);
        }

        self.current_output = Some(OutputSpec {
            path: path.into(),
            options: Vec::new(),
        });
        Ok(self)
    }

    // ==================== Video Codec Options ====================

    /// Set video codec
    pub fn video_codec(mut self, codec: VideoCodec) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-c:v".to_string());
            output.options.push(codec.to_string());
        } else {
            self.post_input_args.push("-c:v".to_string());
            self.post_input_args.push(codec.to_string());
        }
        self
    }

    /// Set video codec with hardware acceleration
    pub fn video_codec_hw(mut self, codec: VideoCodec, hw: HardwareAccel) -> Self {
        let codec_str = match (&codec, &hw) {
            (VideoCodec::H264, HardwareAccel::NVENC) => "h264_nvenc",
            (VideoCodec::H265, HardwareAccel::NVENC) => "hevc_nvenc",
            (VideoCodec::H264, HardwareAccel::QSV) => "h264_qsv",
            (VideoCodec::H265, HardwareAccel::QSV) => "hevc_qsv",
            (VideoCodec::H264, HardwareAccel::VideoToolbox) => "h264_videotoolbox",
            (VideoCodec::H265, HardwareAccel::VideoToolbox) => "hevc_videotoolbox",
            (VideoCodec::H264, HardwareAccel::VAAPI) => "h264_vaapi",
            (VideoCodec::H265, HardwareAccel::VAAPI) => "hevc_vaapi",
            (VideoCodec::H264, HardwareAccel::AMF) => "h264_amf",
            (VideoCodec::H265, HardwareAccel::AMF) => "hevc_amf",
            _ => return self.video_codec(codec), // Fall back to software
        };

        if let Some(output) = &mut self.current_output {
            output.options.push("-c:v".to_string());
            output.options.push(codec_str.to_string());
        } else {
            self.post_input_args.push("-c:v".to_string());
            self.post_input_args.push(codec_str.to_string());
        }
        self
    }

    /// Set constant rate factor (CRF) for video quality (0-51, lower is better)
    pub fn crf(mut self, value: u8) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-crf".to_string());
            output.options.push(value.to_string());
        } else {
            self.post_input_args.push("-crf".to_string());
            self.post_input_args.push(value.to_string());
        }
        self
    }

    /// Set video bitrate
    pub fn video_bitrate(mut self, bitrate: impl Into<String>) -> Result<Self> {
        let bitrate = bitrate.into();
        if let Some(output) = &mut self.current_output {
            output.options.push("-b:v".to_string());
            output.options.push(bitrate);
        } else {
            self.post_input_args.push("-b:v".to_string());
            self.post_input_args.push(bitrate);
        }
        Ok(self)
    }

    /// Set maximum video bitrate
    pub fn max_video_bitrate(mut self, bitrate: impl Into<String>) -> Result<Self> {
        let bitrate = bitrate.into();
        if let Some(output) = &mut self.current_output {
            output.options.push("-maxrate".to_string());
            output.options.push(bitrate);
        } else {
            self.post_input_args.push("-maxrate".to_string());
            self.post_input_args.push(bitrate);
        }
        Ok(self)
    }

    /// Set buffer size for bitrate control
    pub fn buffer_size(mut self, size: impl Into<String>) -> Self {
        let size = size.into();
        if let Some(output) = &mut self.current_output {
            output.options.push("-bufsize".to_string());
            output.options.push(size);
        } else {
            self.post_input_args.push("-bufsize".to_string());
            self.post_input_args.push(size);
        }
        self
    }

    /// Set encoding preset
    pub fn preset(mut self, preset: Preset) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-preset".to_string());
            output.options.push(preset.to_string());
        } else {
            self.post_input_args.push("-preset".to_string());
            self.post_input_args.push(preset.to_string());
        }
        self
    }

    /// Set pixel format
    pub fn pixel_format(mut self, format: PixelFormat) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-pix_fmt".to_string());
            output.options.push(format.to_string());
        } else {
            self.post_input_args.push("-pix_fmt".to_string());
            self.post_input_args.push(format.to_string());
        }
        self
    }

    /// Set output framerate
    pub fn framerate(mut self, fps: impl Into<String>) -> Self {
        let fps = fps.into();
        if let Some(output) = &mut self.current_output {
            output.options.push("-r".to_string());
            output.options.push(fps);
        } else {
            self.post_input_args.push("-r".to_string());
            self.post_input_args.push(fps);
        }
        self
    }

    /// Set keyframe interval (GOP size)
    pub fn keyframe_interval(mut self, frames: u32) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-g".to_string());
            output.options.push(frames.to_string());
        } else {
            self.post_input_args.push("-g".to_string());
            self.post_input_args.push(frames.to_string());
        }
        self
    }

    // ==================== Resolution & Scaling ====================

    /// Set output resolution (e.g., "1920x1080" or "1920:1080")
    pub fn resolution(self, width: u32, height: u32) -> Self {
        let scale = format!("scale={}:{}", width, height);
        self.video_filter(scale)
    }

    /// Set output width (height will be automatically calculated to maintain aspect ratio)
    pub fn width(self, width: u32) -> Self {
        let scale = format!("scale={}:-1", width);
        self.video_filter(scale)
    }

    /// Set output height (width will be automatically calculated to maintain aspect ratio)
    pub fn height(self, height: u32) -> Self {
        let scale = format!("scale=-1:{}", height);
        self.video_filter(scale)
    }

    /// Set scaling algorithm
    pub fn scale_algorithm(mut self, algorithm: ScaleAlgorithm) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-sws_flags".to_string());
            output.options.push(algorithm.to_string());
        } else {
            self.post_input_args.push("-sws_flags".to_string());
            self.post_input_args.push(algorithm.to_string());
        }
        self
    }

    // ==================== Audio Codec Options ====================

    /// Set audio codec
    pub fn audio_codec(mut self, codec: AudioCodec) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-c:a".to_string());
            output.options.push(codec.to_string());
        } else {
            self.post_input_args.push("-c:a".to_string());
            self.post_input_args.push(codec.to_string());
        }
        self
    }

    /// Set audio bitrate
    pub fn audio_bitrate(mut self, bitrate: impl Into<String>) -> Result<Self> {
        let bitrate = bitrate.into();
        if let Some(output) = &mut self.current_output {
            output.options.push("-b:a".to_string());
            output.options.push(bitrate);
        } else {
            self.post_input_args.push("-b:a".to_string());
            self.post_input_args.push(bitrate);
        }
        Ok(self)
    }

    /// Set audio sample rate
    pub fn audio_sample_rate(mut self, rate: u32) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-ar".to_string());
            output.options.push(rate.to_string());
        } else {
            self.post_input_args.push("-ar".to_string());
            self.post_input_args.push(rate.to_string());
        }
        self
    }

    /// Set audio channels
    pub fn audio_channels(mut self, channels: u8) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-ac".to_string());
            output.options.push(channels.to_string());
        } else {
            self.post_input_args.push("-ac".to_string());
            self.post_input_args.push(channels.to_string());
        }
        self
    }

    /// Set audio channel layout
    pub fn channel_layout(mut self, layout: ChannelLayout) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-channel_layout".to_string());
            output.options.push(layout.to_string());
        } else {
            self.post_input_args.push("-channel_layout".to_string());
            self.post_input_args.push(layout.to_string());
        }
        self
    }

    /// Set audio quality (for VBR encoding, 0-10 where lower is better for most codecs)
    pub fn audio_quality(mut self, quality: u8) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-q:a".to_string());
            output.options.push(quality.to_string());
        } else {
            self.post_input_args.push("-q:a".to_string());
            self.post_input_args.push(quality.to_string());
        }
        self
    }

    /// Disable audio
    pub fn no_audio(mut self) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-an".to_string());
        } else {
            self.post_input_args.push("-an".to_string());
        }
        self
    }

    /// Disable video
    pub fn no_video(mut self) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-vn".to_string());
        } else {
            self.post_input_args.push("-vn".to_string());
        }
        self
    }

    /// Disable subtitles
    pub fn no_subtitles(mut self) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-sn".to_string());
        } else {
            self.post_input_args.push("-sn".to_string());
        }
        self
    }

    // ==================== Format Options ====================

    /// Set output format
    pub fn format(mut self, format: ContainerFormat) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-f".to_string());
            output.options.push(format.to_string());
        } else {
            self.post_input_args.push("-f".to_string());
            self.post_input_args.push(format.to_string());
        }
        self
    }

    /// Set movflags for MP4 (e.g., "faststart" for web streaming)
    pub fn movflags(mut self, flags: impl Into<String>) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-movflags".to_string());
            output.options.push(flags.into());
        } else {
            self.post_input_args.push("-movflags".to_string());
            self.post_input_args.push(flags.into());
        }
        self
    }

    // ==================== Filters ====================

    /// Add a video filter
    pub fn video_filter(mut self, filter: impl Into<String>) -> Self {
        let filter = filter.into();
        if let Some(output) = &mut self.current_output {
            // Check if we already have a -vf option
            if let Some(pos) = output.options.iter().position(|s| s == "-vf") {
                // Append to existing filter
                if let Some(existing) = output.options.get_mut(pos + 1) {
                    existing.push_str(&format!(",{}", filter));
                }
            } else {
                output.options.push("-vf".to_string());
                output.options.push(filter);
            }
        } else {
            // Same logic for post_input_args
            if let Some(pos) = self.post_input_args.iter().position(|s| s == "-vf") {
                if let Some(existing) = self.post_input_args.get_mut(pos + 1) {
                    existing.push_str(&format!(",{}", filter));
                }
            } else {
                self.post_input_args.push("-vf".to_string());
                self.post_input_args.push(filter);
            }
        }
        self
    }

    /// Add an audio filter
    pub fn audio_filter(mut self, filter: impl Into<String>) -> Self {
        let filter = filter.into();
        if let Some(output) = &mut self.current_output {
            if let Some(pos) = output.options.iter().position(|s| s == "-af") {
                if let Some(existing) = output.options.get_mut(pos + 1) {
                    existing.push_str(&format!(",{}", filter));
                }
            } else {
                output.options.push("-af".to_string());
                output.options.push(filter);
            }
        } else {
            if let Some(pos) = self.post_input_args.iter().position(|s| s == "-af") {
                if let Some(existing) = self.post_input_args.get_mut(pos + 1) {
                    existing.push_str(&format!(",{}", filter));
                }
            } else {
                self.post_input_args.push("-af".to_string());
                self.post_input_args.push(filter);
            }
        }
        self
    }

    /// Add a complex filter graph
    pub fn filter_complex(mut self, filter: impl Into<String>) -> Self {
        self.post_input_args.push("-filter_complex".to_string());
        self.post_input_args.push(filter.into());
        self
    }

    // ==================== Stream Selection & Mapping ====================

    /// Map a stream from input to output (e.g., "0:v:0" for first video stream of first input)
    pub fn map(mut self, stream: impl Into<String>) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-map".to_string());
            output.options.push(stream.into());
        } else {
            self.post_input_args.push("-map".to_string());
            self.post_input_args.push(stream.into());
        }
        self
    }

    /// Select specific stream by type and index
    pub fn select_stream(self, stream_type: StreamType, input_index: usize, stream_index: usize) -> Self {
        let map_spec = format!("{}:{}:{}", input_index, stream_type, stream_index);
        self.map(map_spec)
    }

    // ==================== Metadata ====================

    /// Set metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-metadata".to_string());
            output.options.push(format!("{}={}", key.into(), value.into()));
        } else {
            self.post_input_args.push("-metadata".to_string());
            self.post_input_args.push(format!("{}={}", key.into(), value.into()));
        }
        self
    }

    // ==================== Two-Pass Encoding ====================

    /// Enable first pass of two-pass encoding
    pub fn two_pass_first(mut self, log_file: impl Into<String>) -> Self {
        let log = log_file.into();
        if let Some(output) = &mut self.current_output {
            output.options.push("-pass".to_string());
            output.options.push("1".to_string());
            output.options.push("-passlogfile".to_string());
            output.options.push(log);
            output.options.push("-f".to_string());
            output.options.push("null".to_string());
        }
        self
    }

    /// Enable second pass of two-pass encoding
    pub fn two_pass_second(mut self, log_file: impl Into<String>) -> Self {
        let log = log_file.into();
        if let Some(output) = &mut self.current_output {
            output.options.push("-pass".to_string());
            output.options.push("2".to_string());
            output.options.push("-passlogfile".to_string());
            output.options.push(log);
        }
        self
    }

    // ==================== Streaming ====================

    /// Configure HLS output
    pub fn hls_options(
        mut self,
        segment_time: u32,
        list_size: u32,
        format: impl Into<String>,
    ) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-f".to_string());
            output.options.push("hls".to_string());
            output.options.push("-hls_time".to_string());
            output.options.push(segment_time.to_string());
            output.options.push("-hls_list_size".to_string());
            output.options.push(list_size.to_string());
            output.options.push("-hls_segment_filename".to_string());
            output.options.push(format.into());
        }
        self
    }

    /// Configure DASH output
    pub fn dash_options(mut self, segment_duration: u32, window_size: u32) -> Self {
        if let Some(output) = &mut self.current_output {
            output.options.push("-f".to_string());
            output.options.push("dash".to_string());
            output.options.push("-seg_duration".to_string());
            output.options.push(segment_duration.to_string());
            output.options.push("-window_size".to_string());
            output.options.push(window_size.to_string());
        }
        self
    }

    // ==================== Build & Execute ====================

    /// Build the command and return an executable command
    pub fn build(mut self) -> Result<FFmpegCommand> {
        // Finalize any pending input/output
        if let Some(input) = self.current_input.take() {
            self.inputs.push(input);
        }
        if let Some(output) = self.current_output.take() {
            self.outputs.push(output);
        }

        // Validate
        if self.inputs.is_empty() {
            return Err(BuilderError::NoInput);
        }
        if self.outputs.is_empty() {
            return Err(BuilderError::NoOutput);
        }

        // Build argument list
        let mut args = Vec::new();

        // Global args first
        args.extend(self.global_args);

        // Pre-input args
        args.extend(self.pre_input_args);

        // Input files with their options
        for input in self.inputs {
            args.extend(input.options);
            args.push("-i".to_string());
            args.push(input.path);
        }

        // Post-input args (codecs, filters, etc.)
        args.extend(self.post_input_args);

        // Output files with their options
        for output in self.outputs {
            args.extend(output.options);
            args.push(output.path);
        }

        Ok(FFmpegCommand {
            ffmpeg: self.ffmpeg,
            args,
        })
    }
}

/// Executable FFmpeg command
pub struct FFmpegCommand {
    ffmpeg: Arc<FFMpeg>,
    args: Vec<String>,
}

impl FFmpegCommand {
    /// Get the command arguments as a vector of strings
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Get the command as a string (for debugging/logging)
    pub fn to_string(&self) -> String {
        format!("ffmpeg {}", self.args.join(" "))
    }

    /// Execute the command
    pub async fn execute(
        &self,
        working_dir: Option<PathBuf>,
        sender: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<()> {
        let working_dir = working_dir.unwrap_or_else(|| PathBuf::from("."));
        let (tx, _rx) = tokio::sync::mpsc::channel(100);
        let sender = sender.unwrap_or(tx);

        let args: Vec<&str> = self.args.iter().map(|s| s.as_str()).collect();
        self.ffmpeg
            .exec_ffmpeg(&args, working_dir, sender)
            .await
            .map_err(|e| BuilderError::ExecutionError(e.to_string()))?;

        Ok(())
    }

    /// Execute the command and wait for completion
    pub async fn execute_blocking(
        &self,
        working_dir: Option<PathBuf>,
    ) -> Result<Vec<String>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let mut output = Vec::new();

        // Spawn execution
        let execute_future = self.execute(working_dir, Some(tx));

        // Collect output
        let collect_future = async {
            while let Some(line) = rx.recv().await {
                output.push(line);
            }
        };

        // Wait for both to complete
        tokio::try_join!(execute_future, async {
            collect_future.await;
            Ok(())
        })?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_basic() {
        let ffmpeg = Arc::new(FFMpeg::default());
        let builder = FFmpegBuilder::new(ffmpeg)
            .input("input.mp4")
            .unwrap()
            .output("output.mp4")
            .unwrap()
            .video_codec(VideoCodec::H264)
            .audio_codec(AudioCodec::AAC);

        let cmd = builder.build().unwrap();
        let args = cmd.args();

        assert!(args.contains(&"-i".to_string()));
        assert!(args.contains(&"input.mp4".to_string()));
        assert!(args.contains(&"-c:v".to_string()));
        assert!(args.contains(&"libx264".to_string()));
        assert!(args.contains(&"-c:a".to_string()));
        assert!(args.contains(&"aac".to_string()));
        assert!(args.contains(&"output.mp4".to_string()));
    }

    #[test]
    fn test_builder_no_input_error() {
        let ffmpeg = Arc::new(FFMpeg::default());
        let builder = FFmpegBuilder::new(ffmpeg).output("output.mp4").unwrap();

        let result = builder.build();
        assert!(matches!(result, Err(BuilderError::NoInput)));
    }

    #[test]
    fn test_builder_no_output_error() {
        let ffmpeg = Arc::new(FFMpeg::default());
        let builder = FFmpegBuilder::new(ffmpeg).input("input.mp4").unwrap();

        let result = builder.build();
        assert!(matches!(result, Err(BuilderError::NoOutput)));
    }
}
