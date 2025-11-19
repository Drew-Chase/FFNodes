use ffmpeg::builders::{FFmpegBuilder, FFprobeBuilder};
use ffmpeg::builders::common::{AudioCodec, HardwareAccel, Preset, ProbeFormat, VideoCodec};
use ffmpeg::FFMpeg;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize FFmpeg (will auto-download if not found)
    let mut ffmpeg = FFMpeg::default();
    ffmpeg.fetch().await?;
    let ffmpeg = Arc::new(ffmpeg);

    println!("=== FFprobe Example: Analyzing video metadata ===");
    probe_example(ffmpeg.clone()).await?;

    println!("\n=== FFmpeg Example 1: Basic video conversion ===");
    basic_conversion_example(ffmpeg.clone()).await?;

    println!("\n=== FFmpeg Example 2: Hardware-accelerated encoding ===");
    hardware_accel_example(ffmpeg.clone()).await?;

    println!("\n=== FFmpeg Example 3: Complex filtering ===");
    filter_example(ffmpeg.clone()).await?;

    println!("\n=== FFmpeg Example 4: HLS streaming output ===");
    streaming_example(ffmpeg.clone()).await?;

    Ok(())
}

async fn probe_example(ffmpeg: Arc<FFMpeg>) -> Result<(), Box<dyn std::error::Error>> {
    // Build FFprobe command to analyze a video file
    let cmd = FFprobeBuilder::new(ffmpeg)
        .input("input.mp4")?
        .show_streams()
        .show_format()
        .output_format(ProbeFormat::JSON)
        .hide_banner()
        .build()?;

    println!("Command: {}", cmd.to_string());

    // Uncomment to execute:
    // let metadata = cmd.execute_json().await?;
    //
    // // Print video information
    // if let Some((width, height)) = metadata.video_resolution() {
    //     println!("Resolution: {}x{}", width, height);
    // }
    // if let Some(duration) = metadata.duration_seconds() {
    //     println!("Duration: {:.2} seconds", duration);
    // }
    // if let Some(bitrate) = metadata.bit_rate() {
    //     println!("Bitrate: {} bps", bitrate);
    // }
    //
    // // Print stream information
    // for stream in metadata.video_streams() {
    //     println!("Video codec: {:?}", stream.codec_name);
    //     println!("Video bitrate: {:?}", stream.bit_rate);
    // }
    // for stream in metadata.audio_streams() {
    //     println!("Audio codec: {:?}", stream.codec_name);
    //     println!("Audio channels: {:?}", stream.channels);
    //     println!("Audio sample rate: {:?}", stream.sample_rate);
    // }

    Ok(())
}

async fn basic_conversion_example(ffmpeg: Arc<FFMpeg>) -> Result<(), Box<dyn std::error::Error>> {
    // Convert video with custom codec settings
    let cmd = FFmpegBuilder::new(ffmpeg)
        .input("input.mp4")?
        .start_time("00:01:30")? // Start at 1 minute 30 seconds
        .duration("00:05:00")? // Take 5 minutes
        .output("output.mp4")?
        .video_codec(VideoCodec::H264)
        .crf(23) // Quality setting (lower = better quality, 18-28 is common)
        .preset(Preset::Fast)
        .audio_codec(AudioCodec::AAC)
        .audio_bitrate("192k")?
        .overwrite(true)
        .hide_banner()
        .build()?;

    println!("Command: {}", cmd.to_string());

    // Uncomment to execute:
    // cmd.execute(None, None).await?;
    // println!("Conversion complete!");

    Ok(())
}

async fn hardware_accel_example(ffmpeg: Arc<FFMpeg>) -> Result<(), Box<dyn std::error::Error>> {
    // Use NVIDIA NVENC for hardware-accelerated encoding
    let cmd = FFmpegBuilder::new(ffmpeg)
        .hardware_accel(HardwareAccel::CUDA)
        .input("input.mp4")?
        .output("output_hw.mp4")?
        .video_codec_hw(VideoCodec::H264, HardwareAccel::NVENC)
        .preset(Preset::Fast)
        .video_bitrate("5M")?
        .audio_codec(AudioCodec::Copy) // Copy audio without re-encoding
        .overwrite(true)
        .build()?;

    println!("Command: {}", cmd.to_string());

    // Uncomment to execute:
    // cmd.execute(None, None).await?;
    // println!("Hardware-accelerated encoding complete!");

    Ok(())
}

async fn filter_example(ffmpeg: Arc<FFMpeg>) -> Result<(), Box<dyn std::error::Error>> {
    // Apply multiple filters to video and audio
    let cmd = FFmpegBuilder::new(ffmpeg)
        .input("input.mp4")?
        .output("filtered.mp4")?
        .video_codec(VideoCodec::H264)
        .width(1920) // Scale to 1920 width, maintaining aspect ratio
        .video_filter("fps=30") // Set framerate to 30fps
        .video_filter("eq=brightness=0.1:contrast=1.2") // Adjust brightness and contrast
        .audio_codec(AudioCodec::AAC)
        .audio_filter("volume=2.0") // Double the volume
        .audio_filter("highpass=f=200") // High-pass filter at 200Hz
        .overwrite(true)
        .build()?;

    println!("Command: {}", cmd.to_string());

    // Uncomment to execute:
    // cmd.execute(None, None).await?;
    // println!("Filtering complete!");

    Ok(())
}

async fn streaming_example(ffmpeg: Arc<FFMpeg>) -> Result<(), Box<dyn std::error::Error>> {
    // Convert video to HLS format for streaming
    let cmd = FFmpegBuilder::new(ffmpeg)
        .input("input.mp4")?
        .output("output.m3u8")?
        .video_codec(VideoCodec::H264)
        .video_bitrate("2M")?
        .audio_codec(AudioCodec::AAC)
        .audio_bitrate("128k")?
        .hls_options(
            6,                   // Segment duration in seconds
            5,                   // Number of segments in playlist
            "segment_%03d.ts",   // Segment filename pattern
        )
        .movflags("faststart") // Optimize for web streaming
        .overwrite(true)
        .build()?;

    println!("Command: {}", cmd.to_string());

    // Uncomment to execute:
    // cmd.execute(None, None).await?;
    // println!("HLS streaming files created!");

    Ok(())
}
