use crate::configuration::Configuration;
use crate::media_files::MediaFile;
use crate::media_files::media_file_db::open_pool;
use anyhow::Result;
use log::{debug, error, info, trace};
use std::path::PathBuf;
use std::sync::Arc;
use walkdir::WalkDir;
use futures::stream::{self, StreamExt};

const VIDEO_EXTENSIONS: [&str; 47] = [
    "webm", "mkv", "flv", "flv", "vob", "ogv", "ogg", "drc", "gif", "gifv", "mng", "avi", "mts",
    "m2ts", "ts", "mov", "qt", "wmv", "yuv", "rm", "rmvb", "viv", "asf", "amv", "mp4", "m4p",
    "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv", "mpg", "mpeg", "m2v", "m4v", "svi", "3gp", "3g2",
    "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b",
];

pub struct Scanner;
impl Scanner {
    pub async fn scan(watch_directories: Vec<PathBuf>, config: Arc<Configuration>) -> Result<()> {
        info!("Scanning files");

        // Open database pool once for all inserts
        let pool = open_pool().await?;

        // Collect all video file paths that need probing
        let mut files: Vec<PathBuf> = vec![];
        for dir in watch_directories {
            debug!("Scanning directory {:?}", dir);
            for entry in WalkDir::new(dir) {
                let entry = entry?;
                if let Some(extension) = entry.path().extension()
                    && VIDEO_EXTENSIONS.contains(&extension.to_string_lossy().to_string().as_str())
                {
                    // Check if file needs probing using smart re-probing logic
                    match MediaFile::does_path_need_probing(entry.path(), &pool).await {
                        Ok(needs_probing) => {
                            if needs_probing {
                                debug!("File needs probing: {:?}", entry.path());
                                files.push(entry.into_path());
                            } else {
                                trace!("File already up-to-date: {:?}", entry.path());
                            }
                        }
                        Err(_) => {
                            // File doesn't exist in database, needs probing
                            debug!("New file found: {:?}", entry.path());
                            files.push(entry.into_path());
                        }
                    }
                }
            }
        }

        // Process and insert files concurrently as they're probed
        // buffer_unordered allows up to 10 concurrent ffprobe operations
        let insert_count = stream::iter(files)
            .map(|file| {
                let config = Arc::clone(&config);
                let pool = pool.clone();
                async move {
                    trace!("Probing video file: {:?}", file);
                    match MediaFile::from_path_with_config(&file, &config, false).await {
                        Ok(media_file) => {
                            // Insert immediately after probing
                            match media_file.insert_direct(&pool).await {
                                Ok(_) => {
                                    debug!("Inserted {:?} into database", file);
                                    1
                                }
                                Err(e) => {
                                    error!("Failed to insert {:?}: {:#}", file, e);
                                    0
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to probe {:?}: {:#}", file, e);
                            0
                        }
                    }
                }
            })
            .buffer_unordered(100)
            .fold(0, |acc, count| async move { acc + count })
            .await;

        info!("Successfully inserted {} media files into database", insert_count);
        Ok(())
    }
}
