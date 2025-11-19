use anyhow::Result;
use serde_hash::HashIds;
use std::path::{Path, PathBuf};

mod media_file_db;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow, HashIds)]
/// A video media file.
pub struct MediaFile {
    #[hash]
    /// The database id of the file.
    pub id: u64,
    /// The path to the file on disk.
    pub path: PathBuf,
    /// The original file size at the point of scanning.
    pub scanned_size: u64,
    /// The current file size, if the file has been processed.
    pub size: Option<u64>,
    /// The bit rate of the file at the point of scanning, if available.
    pub scanned_bit_rate: Option<u64>,
    /// The bit rate of the file, if it has been processed.
    pub bit_rate: Option<u64>,
    /// The mime type of the file.
    pub mime_type: String,
    /// The duration of the file, if it's available.
    pub duration: Option<u64>,
    /// The width and height of the file, if it's available.
    pub width: Option<u64>,
    /// The width and height of the file, if it's available.
    pub height: Option<u64>,
    /// Whether the file has been processed.
    pub processed: bool,
}

impl MediaFile {
    pub async fn new(file_path: impl AsRef<Path>) -> Result<Self> {
        unimplemented!()
    }
}