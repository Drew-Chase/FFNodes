# FFmpeg & FFprobe Command Builders

Comprehensive, type-safe, fluent API builders for FFmpeg and FFprobe commands in Rust.

## Features

### FFmpegBuilder

A comprehensive command builder that covers most FFmpeg options:

#### Core Features
- **Input/Output Management**: Multiple inputs/outputs with per-file options
- **Video Codecs**: H.264, H.265, VP9, AV1, and custom codecs
- **Audio Codecs**: AAC, Opus, MP3, FLAC, Vorbis, and custom codecs
- **Quality Control**: CRF, bitrate (VBR/CBR), presets
- **Resolution & Scaling**: Fixed resolution, width/height scaling with aspect ratio preservation
- **Hardware Acceleration**: NVENC, QSV, VideoToolbox, VAAPI, AMF, CUDA
- **Filters**: Video/audio filters with automatic chaining, complex filter graphs
- **Two-Pass Encoding**: Full support for multi-pass workflows
- **Streaming Protocols**: HLS, DASH configuration
- **Stream Mapping**: Select and map specific streams
- **Metadata**: Set custom metadata tags
- **Raw Args**: Escape hatch for unsupported options

#### Builder Pattern
The builder uses a fluent API with method chaining:
```rust
FFmpegBuilder::new(ffmpeg)
    .input("input.mp4")?
        .start_time("00:01:30")?
        .duration("00:05:00")?
    .output("output.mp4")?
        .video_codec(VideoCodec::H264)
        .crf(23)
        .preset(Preset::Fast)
        .audio_codec(AudioCodec::AAC)
        .audio_bitrate("192k")?
    .overwrite(true)
    .build()?
```

### FFprobeBuilder

A builder for FFprobe commands with JSON parsing support:

#### Core Features
- **Stream Information**: Video, audio, subtitle stream details
- **Format Information**: Container format, duration, bitrate
- **Chapter Information**: Chapter markers and metadata
- **Multiple Output Formats**: JSON, XML, CSV, flat, INI
- **Typed JSON Parsing**: Strongly-typed Rust structures for JSON output
- **Helper Methods**: Easy access to common metadata (resolution, duration, bitrate)

#### Builder Pattern
```rust
FFprobeBuilder::new(ffmpeg)
    .input("video.mp4")?
    .show_streams()
    .show_format()
    .output_format(ProbeFormat::JSON)
    .build()?
```

## Usage Examples

### Basic Video Conversion

```rust
use ffmpeg::builders::FFmpegBuilder;
use ffmpeg::builders::common::{VideoCodec, AudioCodec, Preset};
use ffmpeg::FFMpeg;
use std::sync::Arc;

let mut ffmpeg = FFMpeg::default();
ffmpeg.fetch().await?;
let ffmpeg = Arc::new(ffmpeg);

let cmd = FFmpegBuilder::new(ffmpeg)
    .input("input.mp4")?
    .output("output.mp4")?
        .video_codec(VideoCodec::H264)
        .crf(23)
        .preset(Preset::Fast)
        .audio_codec(AudioCodec::AAC)
        .audio_bitrate("192k")?
    .overwrite(true)
    .build()?;

cmd.execute(None, None).await?;
```

### Probing Video Metadata

```rust
use ffmpeg::builders::FFprobeBuilder;
use ffmpeg::builders::common::ProbeFormat;

let cmd = FFprobeBuilder::new(ffmpeg)
    .input("video.mp4")?
    .show_streams()
    .show_format()
    .output_format(ProbeFormat::JSON)
    .build()?;

let metadata = cmd.execute_json().await?;

// Access metadata
if let Some((width, height)) = metadata.video_resolution() {
    println!("Resolution: {}x{}", width, height);
}
if let Some(duration) = metadata.duration_seconds() {
    println!("Duration: {:.2}s", duration);
}

// Access streams
for stream in metadata.video_streams() {
    println!("Video codec: {:?}", stream.codec_name);
}
```

### Hardware-Accelerated Encoding

```rust
use ffmpeg::builders::common::{VideoCodec, HardwareAccel};

let cmd = FFmpegBuilder::new(ffmpeg)
    .hardware_accel(HardwareAccel::CUDA)
    .input("input.mp4")?
    .output("output.mp4")?
        .video_codec_hw(VideoCodec::H264, HardwareAccel::NVENC)
        .preset(Preset::Fast)
        .video_bitrate("5M")?
    .overwrite(true)
    .build()?;

cmd.execute(None, None).await?;
```

### Complex Filtering

```rust
let cmd = FFmpegBuilder::new(ffmpeg)
    .input("input.mp4")?
    .output("output.mp4")?
        .video_codec(VideoCodec::H264)
        .width(1920)  // Scale to 1920 width
        .video_filter("fps=30")  // Set framerate
        .video_filter("eq=brightness=0.1:contrast=1.2")  // Adjust colors
        .audio_filter("volume=2.0")  // Double volume
    .overwrite(true)
    .build()?;

cmd.execute(None, None).await?;
```

### Two-Pass Encoding

```rust
// First pass
let cmd1 = FFmpegBuilder::new(ffmpeg.clone())
    .input("input.mp4")?
    .output("/dev/null")?
        .video_codec(VideoCodec::H264)
        .video_bitrate("2M")?
        .two_pass_first("passlog")
    .overwrite(true)
    .build()?;
cmd1.execute(None, None).await?;

// Second pass
let cmd2 = FFmpegBuilder::new(ffmpeg)
    .input("input.mp4")?
    .output("output.mp4")?
        .video_codec(VideoCodec::H264)
        .video_bitrate("2M")?
        .two_pass_second("passlog")
    .overwrite(true)
    .build()?;
cmd2.execute(None, None).await?;
```

### HLS Streaming

```rust
let cmd = FFmpegBuilder::new(ffmpeg)
    .input("input.mp4")?
    .output("output.m3u8")?
        .video_codec(VideoCodec::H264)
        .video_bitrate("2M")?
        .audio_codec(AudioCodec::AAC)
        .hls_options(
            6,                   // Segment duration (seconds)
            5,                   // Playlist size
            "segment_%03d.ts",   // Segment filename pattern
        )
        .movflags("faststart")
    .overwrite(true)
    .build()?;

cmd.execute(None, None).await?;
```

### Stream Mapping

```rust
// Copy specific streams from input to output
let cmd = FFmpegBuilder::new(ffmpeg)
    .input("input.mkv")?
    .output("output.mp4")?
        .map("0:v:0")  // First video stream
        .map("0:a:1")  // Second audio stream
        .video_codec(VideoCodec::Copy)
        .audio_codec(AudioCodec::Copy)
    .overwrite(true)
    .build()?;
```

### Multiple Inputs/Outputs

```rust
let cmd = FFmpegBuilder::new(ffmpeg)
    .input("video.mp4")?
    .input("audio.mp3")?
    .output("combined.mp4")?
        .video_codec(VideoCodec::Copy)
        .audio_codec(AudioCodec::AAC)
    .overwrite(true)
    .build()?;
```

### Using Raw Arguments

For options not yet supported by the builder:

```rust
let cmd = FFmpegBuilder::new(ffmpeg)
    .input("input.mp4")?
    .output("output.mp4")?
        .video_codec(VideoCodec::H264)
        .raw_arg("-tune")
        .raw_arg("film")
        .raw_arg("-x264-params")
        .raw_arg("ref=4:bframes=3")
    .overwrite(true)
    .build()?;
```

## Type-Safe Enums

The builders use enums for type safety:

- `VideoCodec`: H264, H265, VP9, AV1, Copy, Custom(String)
- `AudioCodec`: AAC, Opus, MP3, FLAC, Vorbis, Copy, Custom(String)
- `Preset`: UltraFast, SuperFast, VeryFast, Faster, Fast, Medium, Slow, Slower, VerySlow
- `PixelFormat`: YUV420P, YUV422P, YUV444P, RGB24, RGBA, Gray, Custom(String)
- `HardwareAccel`: None, NVENC, QSV, VideoToolbox, VAAPI, AMF, CUDA, Custom(String)
- `StreamType`: Video, Audio, Subtitle, Data, Attachment
- `ChannelLayout`: Mono, Stereo, Surround2_1, Surround5_1, Surround7_1, Custom(String)
- `ScaleAlgorithm`: FastBilinear, Bilinear, Bicubic, Lanczos, etc.
- `ContainerFormat`: MP4, MKV, WebM, AVI, MOV, FLV, MPEG, HLS, DASH, Custom(String)
- `ProbeFormat`: Default, JSON, XML, CSV, Flat, Ini

## Error Handling

All builders use a custom `BuilderError` type with comprehensive error cases:

```rust
use ffmpeg::builders::error::BuilderError;

match builder.build() {
    Ok(cmd) => { /* ... */ },
    Err(BuilderError::NoInput) => eprintln!("No input file specified"),
    Err(BuilderError::NoOutput) => eprintln!("No output file specified"),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Running the Example

```bash
cargo run --example builder_demo
```

## Architecture

The builder pattern is based on the C# reference implementation but adapted for idiomatic Rust:

1. **Fluent API**: All methods return `self` for chaining
2. **Type Safety**: Enums instead of strings where appropriate
3. **Escape Hatches**: `raw_arg()` methods for unsupported options
4. **Validation**: Compile-time and runtime checks (e.g., input/output required)
5. **Async Execution**: Built-in async execution with output streaming
6. **JSON Parsing**: Strongly-typed structures for FFprobe JSON output

## Comparison with C# Implementation

| Feature | C# Version | Rust Version |
|---------|-----------|--------------|
| Builder Pattern | ✓ | ✓ |
| Method Chaining | ✓ | ✓ |
| Input/Output Options | ✓ | ✓ |
| Codec Selection | String-based | Type-safe enums + escape hatch |
| Hardware Accel | Limited | Comprehensive (NVENC, QSV, etc.) |
| Filters | Basic | Advanced (auto-chaining, complex graphs) |
| Two-Pass Encoding | ✗ | ✓ |
| Streaming Protocols | ✗ | ✓ (HLS, DASH) |
| FFprobe Support | Separate | Integrated with JSON parsing |
| Error Handling | Exceptions | Result types |
| Async Execution | Callbacks | Tokio async/await |

## Future Enhancements

Potential additions for future versions:

- [ ] Builder for complex filter graphs with visual representation
- [ ] Preset configurations (web streaming, archive, etc.)
- [ ] Progress monitoring with detailed statistics
- [ ] Validation of codec compatibility
- [ ] Automatic quality/size optimization
- [ ] Batch processing support
- [ ] GPU selection for multi-GPU systems
