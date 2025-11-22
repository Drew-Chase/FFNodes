use crate::configuration::Configuration;
use crate::jobs::JobQueue;
use crate::media_files::MediaFile;
use anyhow::Result;
use log::{debug, error, info, warn};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher};
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

const VIDEO_EXTENSIONS: [&str; 47] = [
    "webm", "mkv", "flv", "flv", "vob", "ogv", "ogg", "drc", "gif", "gifv", "mng", "avi", "mts",
    "m2ts", "ts", "mov", "qt", "wmv", "yuv", "rm", "rmvb", "viv", "asf", "amv", "mp4", "m4p",
    "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv", "mpg", "mpeg", "m2v", "m4v", "svi", "3gp", "3g2",
    "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b",
];

pub struct FileWatcher {
    config: Arc<Configuration>,
    pool: SqlitePool,
    job_queue: Arc<JobQueue>,
}

impl FileWatcher {
    pub fn new(config: Arc<Configuration>, pool: SqlitePool, job_queue: Arc<JobQueue>) -> Self {
        Self {
            config,
            pool,
            job_queue,
        }
    }

    /// Start the file watcher
    pub async fn start(self: Arc<Self>) -> Result<()> {
        info!("Starting file watcher");

        let (tx, mut rx) = mpsc::channel(100);

        // Setup notify watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.blocking_send(event);
                }
            },
            Config::default(),
        )?;

        // Watch all configured directories
        for dir in &self.config.watch_directories {
            info!("Watching directory: {:?}", dir);
            watcher.watch(dir, RecursiveMode::Recursive)?;
        }

        // Batch processing
        let batch_interval = Duration::from_secs(self.config.notify_batch_interval_seconds);
        let mut batch_timer = interval(batch_interval);
        let mut pending_events: Vec<Event> = Vec::new();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // Collect events
                    Some(event) = rx.recv() => {
                        pending_events.push(event);
                    }
                    // Process batch
                    _ = batch_timer.tick() => {
                        if !pending_events.is_empty() {
                            debug!("Processing {} file events", pending_events.len());
                            if let Err(e) = self.process_events(&pending_events).await {
                                error!("Error processing file events: {:#}", e);
                            }
                            pending_events.clear();
                        }
                    }
                }
            }
        });

        // Keep watcher alive
        std::mem::forget(watcher);

        Ok(())
    }

    /// Process batched file events
    async fn process_events(&self, events: &[Event]) -> Result<()> {
        for event in events {
            match event.kind {
                EventKind::Create(_) => {
                    for path in &event.paths {
                        if self.is_video_file(path) {
                            info!("New video file detected: {:?}", path);
                            if let Err(e) = self.handle_new_file(path).await {
                                error!("Error handling new file {:?}: {:#}", path, e);
                            }
                        }
                    }
                }
                EventKind::Remove(_) => {
                    for path in &event.paths {
                        if self.is_video_file(path) {
                            info!("Video file removed: {:?}", path);
                            if let Err(e) = self.handle_removed_file(path).await {
                                error!("Error handling removed file {:?}: {:#}", path, e);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Handle new file detection
    async fn handle_new_file(&self, path: &Path) -> Result<()> {
        // Check if we need to probe this file
        let needs_probing = MediaFile::does_path_need_probing(path, &self.pool).await?;

        if !needs_probing {
            debug!("File {:?} already probed, skipping", path);
            return Ok(());
        }

        // Probe the file
        let media_file = MediaFile::from_path_with_config(path, &self.config, false).await?;

        // Insert into database
        media_file.insert_direct(&self.pool).await?;

        // Create encoding job
        let priority = media_file.scanned_size as i64 * media_file.encoding_complexity as i64;
        let job = self
            .job_queue
            .create_job(media_file.path.to_string_lossy().to_string(), priority)
            .await?;

        info!("Created encoding job for {:?}: {}", path, job.id);

        Ok(())
    }

    /// Handle file removal
    async fn handle_removed_file(&self, path: &Path) -> Result<()> {
        // Remove from database (CASCADE will remove related jobs)
        sqlx::query(r#"DELETE FROM media_files WHERE path = ?"#)
            .bind(path.to_string_lossy().to_string())
            .execute(&self.pool)
            .await?;

        info!("Removed file from database: {:?}", path);

        Ok(())
    }

    /// Check if path is a video file
    fn is_video_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            let ext = extension.to_string_lossy().to_lowercase();
            VIDEO_EXTENSIONS.contains(&ext.as_str())
        } else {
            false
        }
    }
}
