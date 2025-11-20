use crate::configuration::Configuration;
use crate::media_files::MediaFile;
use crate::media_files::media_file_db::open_pool;
use anyhow::Result;
use log::{debug, info, trace};
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
        let files = Self::find_media_files(watch_directories, config).await?;
        let pool = open_pool().await?;
        let mut transaction = pool.begin().await?;
        for file in files {
            file.insert(&mut transaction).await?;
        }
        transaction.commit().await?;

        Ok(())
    }

    pub async fn find_media_files(dirs: Vec<PathBuf>, config: Arc<Configuration>) -> Result<Vec<MediaFile>> {
        let mut files: Vec<PathBuf> = vec![];

        for dir in dirs {
            debug!("Scanning directory {:?}", dir);
            for entry in WalkDir::new(dir) {
                let entry = entry?;
                if let Some(extension) = entry.path().extension()
                    && VIDEO_EXTENSIONS.contains(&extension.to_string_lossy().to_string().as_str())
                {
                    debug!("Found video file: {:?}", entry.path());
                    files.push(entry.into_path());
                }
            }
        }

        // Process files concurrently using Tokio's async runtime
        // buffer_unordered allows up to 10 concurrent ffprobe operations
        let items: Vec<MediaFile> = stream::iter(files)
            .map(|file| {
                let config = Arc::clone(&config);
                async move {
                    trace!("Probing video file: {:?}", file);
                    MediaFile::from_path_with_config(&file, &config).await.ok()
                }
            })
            .buffer_unordered(10)
            .filter_map(|result| async move { result })
            .collect()
            .await;

        info!("Found {} media files", items.len());
        Ok(items)
    }
}
