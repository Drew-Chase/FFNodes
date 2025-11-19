use std::fmt;

/// Video codec selection
#[derive(Debug, Clone, PartialEq)]
pub enum VideoCodec {
    /// H.264/AVC codec
    H264,
    /// H.265/HEVC codec
    H265,
    /// VP9 codec
    VP9,
    /// AV1 codec
    AV1,
    /// Copy stream without re-encoding
    Copy,
    /// Custom codec string
    Custom(String),
}

impl fmt::Display for VideoCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VideoCodec::H264 => write!(f, "libx264"),
            VideoCodec::H265 => write!(f, "libx265"),
            VideoCodec::VP9 => write!(f, "libvpx-vp9"),
            VideoCodec::AV1 => write!(f, "libaom-av1"),
            VideoCodec::Copy => write!(f, "copy"),
            VideoCodec::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Audio codec selection
#[derive(Debug, Clone, PartialEq)]
pub enum AudioCodec {
    /// AAC codec
    AAC,
    /// Opus codec
    Opus,
    /// MP3 codec
    MP3,
    /// FLAC codec
    FLAC,
    /// Vorbis codec
    Vorbis,
    /// Copy stream without re-encoding
    Copy,
    /// Custom codec string
    Custom(String),
}

impl fmt::Display for AudioCodec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioCodec::AAC => write!(f, "aac"),
            AudioCodec::Opus => write!(f, "libopus"),
            AudioCodec::MP3 => write!(f, "libmp3lame"),
            AudioCodec::FLAC => write!(f, "flac"),
            AudioCodec::Vorbis => write!(f, "libvorbis"),
            AudioCodec::Copy => write!(f, "copy"),
            AudioCodec::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Encoding preset (speed vs compression efficiency)
#[derive(Debug, Clone, PartialEq)]
pub enum Preset {
    UltraFast,
    SuperFast,
    VeryFast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    VerySlow,
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Preset::UltraFast => write!(f, "ultrafast"),
            Preset::SuperFast => write!(f, "superfast"),
            Preset::VeryFast => write!(f, "veryfast"),
            Preset::Faster => write!(f, "faster"),
            Preset::Fast => write!(f, "fast"),
            Preset::Medium => write!(f, "medium"),
            Preset::Slow => write!(f, "slow"),
            Preset::Slower => write!(f, "slower"),
            Preset::VerySlow => write!(f, "veryslow"),
        }
    }
}

/// Pixel format for video
#[derive(Debug, Clone, PartialEq)]
pub enum PixelFormat {
    YUV420P,
    YUV422P,
    YUV444P,
    RGB24,
    RGBA,
    Gray,
    Custom(String),
}

impl fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PixelFormat::YUV420P => write!(f, "yuv420p"),
            PixelFormat::YUV422P => write!(f, "yuv422p"),
            PixelFormat::YUV444P => write!(f, "yuv444p"),
            PixelFormat::RGB24 => write!(f, "rgb24"),
            PixelFormat::RGBA => write!(f, "rgba"),
            PixelFormat::Gray => write!(f, "gray"),
            PixelFormat::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Hardware acceleration type
#[derive(Debug, Clone, PartialEq)]
pub enum HardwareAccel {
    /// No hardware acceleration
    None,
    /// NVIDIA NVENC
    NVENC,
    /// Intel Quick Sync Video
    QSV,
    /// Apple VideoToolbox
    VideoToolbox,
    /// Video Acceleration API (Linux)
    VAAPI,
    /// AMD Advanced Media Framework
    AMF,
    /// CUDA
    CUDA,
    /// Custom hardware acceleration
    Custom(String),
}

impl fmt::Display for HardwareAccel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HardwareAccel::None => write!(f, "none"),
            HardwareAccel::NVENC => write!(f, "nvenc"),
            HardwareAccel::QSV => write!(f, "qsv"),
            HardwareAccel::VideoToolbox => write!(f, "videotoolbox"),
            HardwareAccel::VAAPI => write!(f, "vaapi"),
            HardwareAccel::AMF => write!(f, "amf"),
            HardwareAccel::CUDA => write!(f, "cuda"),
            HardwareAccel::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Stream type selection
#[derive(Debug, Clone, PartialEq)]
pub enum StreamType {
    Video,
    Audio,
    Subtitle,
    Data,
    Attachment,
}

impl fmt::Display for StreamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamType::Video => write!(f, "v"),
            StreamType::Audio => write!(f, "a"),
            StreamType::Subtitle => write!(f, "s"),
            StreamType::Data => write!(f, "d"),
            StreamType::Attachment => write!(f, "t"),
        }
    }
}

/// Audio channel layout
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelLayout {
    Mono,
    Stereo,
    Surround2_1,
    Surround5_1,
    Surround7_1,
    Custom(String),
}

impl fmt::Display for ChannelLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelLayout::Mono => write!(f, "mono"),
            ChannelLayout::Stereo => write!(f, "stereo"),
            ChannelLayout::Surround2_1 => write!(f, "2.1"),
            ChannelLayout::Surround5_1 => write!(f, "5.1"),
            ChannelLayout::Surround7_1 => write!(f, "7.1"),
            ChannelLayout::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Video scaling algorithm
#[derive(Debug, Clone, PartialEq)]
pub enum ScaleAlgorithm {
    FastBilinear,
    Bilinear,
    Bicubic,
    Experimental,
    Neighbor,
    Area,
    Bicublin,
    Gauss,
    Sinc,
    Lanczos,
    Spline,
}

impl fmt::Display for ScaleAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaleAlgorithm::FastBilinear => write!(f, "fast_bilinear"),
            ScaleAlgorithm::Bilinear => write!(f, "bilinear"),
            ScaleAlgorithm::Bicubic => write!(f, "bicubic"),
            ScaleAlgorithm::Experimental => write!(f, "experimental"),
            ScaleAlgorithm::Neighbor => write!(f, "neighbor"),
            ScaleAlgorithm::Area => write!(f, "area"),
            ScaleAlgorithm::Bicublin => write!(f, "bicublin"),
            ScaleAlgorithm::Gauss => write!(f, "gauss"),
            ScaleAlgorithm::Sinc => write!(f, "sinc"),
            ScaleAlgorithm::Lanczos => write!(f, "lanczos"),
            ScaleAlgorithm::Spline => write!(f, "spline"),
        }
    }
}

/// Container format
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerFormat {
    MP4,
    MKV,
    WebM,
    AVI,
    MOV,
    FLV,
    MPEG,
    HLS,
    DASH,
    Custom(String),
}

impl fmt::Display for ContainerFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContainerFormat::MP4 => write!(f, "mp4"),
            ContainerFormat::MKV => write!(f, "matroska"),
            ContainerFormat::WebM => write!(f, "webm"),
            ContainerFormat::AVI => write!(f, "avi"),
            ContainerFormat::MOV => write!(f, "mov"),
            ContainerFormat::FLV => write!(f, "flv"),
            ContainerFormat::MPEG => write!(f, "mpeg"),
            ContainerFormat::HLS => write!(f, "hls"),
            ContainerFormat::DASH => write!(f, "dash"),
            ContainerFormat::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// FFprobe output format
#[derive(Debug, Clone, PartialEq)]
pub enum ProbeFormat {
    Default,
    JSON,
    XML,
    CSV,
    Flat,
    Ini,
}

impl fmt::Display for ProbeFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeFormat::Default => write!(f, "default"),
            ProbeFormat::JSON => write!(f, "json"),
            ProbeFormat::XML => write!(f, "xml"),
            ProbeFormat::CSV => write!(f, "csv"),
            ProbeFormat::Flat => write!(f, "flat"),
            ProbeFormat::Ini => write!(f, "ini"),
        }
    }
}
