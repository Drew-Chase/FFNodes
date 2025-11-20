use crate::media_files::MediaFile;
use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use walkdir::WalkDir;
use crate::media_files::media_file_db::open_pool;

const VIDEO_EXTENSIONS: [&str; 47] = [
    "webm", "mkv", "flv", "flv", "vob", "ogv", "ogg", "drc", "gif", "gifv", "mng", "avi", "mts",
    "m2ts", "ts", "mov", "qt", "wmv", "yuv", "rm", "rmvb", "viv", "asf", "amv", "mp4", "m4p",
    "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv", "mpg", "mpeg", "m2v", "m4v", "svi", "3gp", "3g2",
    "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b",
];

pub struct Scanner;
impl Scanner {
    pub async fn scan(watch_directories: Vec<PathBuf>) -> Result<()> {
        let files = Self::find_media_files(watch_directories).await?;
        let pool = open_pool().await?;
        let mut transaction = pool.begin().await?;
        for file in files {
            file.insert(&mut transaction).await?;
        }
        transaction.commit().await?;

        Ok(())
    }

    pub async fn find_media_files(dirs: Vec<PathBuf>) -> Result<Vec<MediaFile>> {
        let mut files: Vec<PathBuf> = vec![];

        for dir in dirs {
            for entry in WalkDir::new(dir) {
                let entry = entry?;
                if let Some(extension) = entry.path().extension()
                    && VIDEO_EXTENSIONS.contains(&extension.to_string_lossy().to_string().as_str())
                {
                    files.push(entry.into_path());
                }
            }
        }

        let items: Vec<MediaFile> = files
            .into_par_iter()
            .filter_map(|file| {
                tokio::runtime::Handle::current()
                    .block_on(async { MediaFile::new(&file).await.ok() })
            })
            .collect();

        Ok(items)
    }
}
