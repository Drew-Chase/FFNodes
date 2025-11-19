use thiserror::Error;

/// Errors that can occur when building or executing FFmpeg/FFprobe commands
#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("No input file specified")]
    NoInput,

    #[error("No output file specified")]
    NoOutput,

    #[error("Invalid option: {0}")]
    InvalidOption(String),

    #[error("Incompatible options: {0}")]
    IncompatibleOptions(String),

    #[error("Invalid time format: {0}")]
    InvalidTimeFormat(String),

    #[error("Invalid resolution format: {0}. Expected format like '1920x1080' or '1920:1080'")]
    InvalidResolution(String),

    #[error("Invalid bitrate format: {0}. Expected format like '2M' or '128k'")]
    InvalidBitrate(String),

    #[error("Hardware acceleration not available: {0}")]
    HardwareAccelNotAvailable(String),

    #[error("Filter error: {0}")]
    FilterError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),

    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, BuilderError>;
