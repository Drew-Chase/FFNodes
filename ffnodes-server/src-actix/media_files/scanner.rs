use crate::configuration::Configuration;
use crate::jobs::JobQueue;
use crate::media_files::MediaFile;
use crate::media_files::media_file_db::open_pool;
use crate::media_files::progress::{self, ScanProgress};
use anyhow::Result;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, error, info, trace};
use walkdir::WalkDir;

const VIDEO_EXTENSIONS: [&str; 47] = [
    "webm", "mkv", "flv", "flv", "vob", "ogv", "ogg", "drc", "gif", "gifv", "mng", "avi", "mts",
    "m2ts", "ts", "mov", "qt", "wmv", "yuv", "rm", "rmvb", "viv", "asf", "amv", "mp4", "m4p",
    "m4v", "mpg", "mp2", "mpeg", "mpe", "mpv", "mpg", "mpeg", "m2v", "m4v", "svi", "3gp", "3g2",
    "mxf", "roq", "nsv", "flv", "f4v", "f4p", "f4a", "f4b",
];

pub struct Scanner;
impl Scanner {
    pub async fn scan(
        watch_directories: Vec<PathBuf>,
        config: Arc<Configuration>,
        job_queue: Arc<JobQueue>,
    ) -> Result<()> {
        info!("Scanning files");

        // Broadcast scan start
        progress::send_progress(ScanProgress::new("Scanning directories"));

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

        let total_files = files.len();
        info!("Found {} files to process", total_files);

        if total_files == 0 {
            progress::send_progress(ScanProgress {
                total_files: 0,
                completed_files: 0,
                current_file: None,
                operation: "Complete".to_string(),
            });
            return Ok(());
        }

        // Broadcast total count
        progress::send_progress(ScanProgress::with_total("Probing files", total_files));

        // Create progress bar
        let pb = ProgressBar::new(total_files as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .expect("Invalid progress bar template")
                .progress_chars("█▓▒░ "),
        );

        // Atomic counter for completed files
        let completed = Arc::new(AtomicUsize::new(0));

        // Process and insert files concurrently as they're probed
        // buffer_unordered allows up to 100 concurrent ffprobe operations
        let insert_count = stream::iter(files)
            .map(|file| {
                let config = Arc::clone(&config);
                let pool = pool.clone();
                let job_queue = Arc::clone(&job_queue);
                let pb = pb.clone();
                let completed = Arc::clone(&completed);
                async move {
                    // Get filename for display
                    let filename = file
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Truncate filename if too long
                    let display_name = if filename.len() > 50 {
                        format!("{}...", &filename[..47])
                    } else {
                        filename.clone()
                    };

                    // Update progress bar message
                    pb.set_message(format!(
                        "{} ({:.1}%)",
                        display_name,
                        (completed.load(Ordering::Relaxed) as f64 / total_files as f64) * 100.0
                    ));

                    // Broadcast current file
                    progress::send_progress(ScanProgress {
                        total_files,
                        completed_files: completed.load(Ordering::Relaxed),
                        current_file: Some(file.to_string_lossy().to_string()),
                        operation: "Probing files".to_string(),
                    });

                    trace!("Probing video file: {:?}", file);
                    let result = match MediaFile::from_path_with_config(&file, &config).await {
                        Ok(media_file) => {
                            // Insert immediately after probing
                            match media_file.insert_direct(&pool).await {
                                Ok(_) => {
                                    debug!("Inserted {:?} into database", file);

                                    // Create encoding job for this file
                                    let priority = (media_file.scanned_size as i64)
                                        .saturating_add(media_file.encoding_complexity as i64);
                                    match job_queue
                                        .create_job(
                                            media_file.path.to_string_lossy().to_string(),
                                            priority,
                                        )
                                        .await
                                    {
                                        Ok(job) => {
                                            info!(
                                                "Created encoding job for {:?}: {}",
                                                file, job.id
                                            );
                                            1
                                        }
                                        Err(e) => {
                                            error!("Failed to create job for {:?}: {:#}", file, e);
                                            0
                                        }
                                    }
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
                    };

                    // Update counters
                    completed.fetch_add(1, Ordering::Relaxed);
                    pb.inc(1);

                    result
                }
            })
            .buffer_unordered(100)
            .fold(0, |acc, count| async move { acc + count })
            .await;

        // Finish progress bar and clear it
        pb.finish_and_clear();

        // Broadcast completion
        progress::send_progress(ScanProgress {
            total_files,
            completed_files: total_files,
            current_file: None,
            operation: "Complete".to_string(),
        });

        info!(
            "Successfully inserted {} media files into database",
            insert_count
        );
        Ok(())
    }
}
