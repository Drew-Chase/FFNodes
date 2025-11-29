use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tokio::sync::broadcast;

/// Progress information for media file scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    /// Total number of files to process
    pub total_files: usize,
    /// Number of files completed
    pub completed_files: usize,
    /// Current file being processed (if any)
    pub current_file: Option<String>,
    /// Current operation description
    pub operation: String,
}

impl ScanProgress {
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            total_files: 0,
            completed_files: 0,
            current_file: None,
            operation: operation.into(),
        }
    }

    pub fn with_total(operation: impl Into<String>, total: usize) -> Self {
        Self {
            total_files: total,
            completed_files: 0,
            current_file: None,
            operation: operation.into(),
        }
    }

    #[allow(dead_code)]
    pub fn percentage(&self) -> f64 {
        if self.total_files == 0 {
            0.0
        } else {
            (self.completed_files as f64 / self.total_files as f64) * 100.0
        }
    }
}

/// Global broadcaster for scan progress
static PROGRESS_BROADCASTER: Mutex<Option<broadcast::Sender<ScanProgress>>> = Mutex::new(None);

/// Progress broadcaster handle
pub struct ProgressBroadcaster {
    sender: broadcast::Sender<ScanProgress>,
}

impl ProgressBroadcaster {
    /// Create a new progress broadcaster
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Send a progress update
    pub fn send(&self, progress: ScanProgress) {
        // Ignore errors if no receivers
        let _ = self.sender.send(progress);
    }

    /// Get a receiver for progress updates
    pub fn subscribe(&self) -> broadcast::Receiver<ScanProgress> {
        self.sender.subscribe()
    }

    /// Get a clone of the sender
    pub fn sender(&self) -> broadcast::Sender<ScanProgress> {
        self.sender.clone()
    }
}

impl Clone for ProgressBroadcaster {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

/// Initialize the global progress broadcaster
pub fn init_broadcaster(capacity: usize) -> ProgressBroadcaster {
    let broadcaster = ProgressBroadcaster::new(capacity);
    let mut global = PROGRESS_BROADCASTER.lock().unwrap();
    *global = Some(broadcaster.sender());
    broadcaster
}

/// Get the global progress broadcaster
pub fn get_broadcaster() -> Option<ProgressBroadcaster> {
    let global = PROGRESS_BROADCASTER.lock().unwrap();
    global
        .as_ref()
        .map(|sender| ProgressBroadcaster {
            sender: sender.clone(),
        })
}

/// Send progress to the global broadcaster
pub fn send_progress(progress: ScanProgress) {
    if let Some(broadcaster) = get_broadcaster() {
        broadcaster.send(progress);
    }
}

/// Subscribe to progress updates from the global broadcaster
pub fn subscribe() -> Option<broadcast::Receiver<ScanProgress>> {
    get_broadcaster().map(|b| b.subscribe())
}
