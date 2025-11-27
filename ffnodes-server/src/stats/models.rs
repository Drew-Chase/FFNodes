use serde::{Deserialize, Serialize};

/// Job history entry for a single completed job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobHistoryEntry {
    pub filename: String,
    pub average_speed: f64,
    pub duration_seconds: i64,
    pub size_before: i64,
    pub size_after: i64,
    pub size_saved: i64,
    pub size_reduction_percent: f64,
    pub completed_at: i64,
}

/// Overall statistics for a client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverallStats {
    pub average_speed: f64,
    pub average_duration_seconds: f64,
    pub average_size_reduction_percent: f64,
    pub total_jobs: i64,
    pub total_size_saved: i64,
}

/// Client history response
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientHistoryResponse {
    pub jobs: Vec<JobHistoryEntry>,
    pub overall: OverallStats,
}

/// Active job progress for remote users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteJobProgress {
    pub client_name: String,
    pub filename: String,
    pub percentage: f64,
    pub speed: f64,
    pub frame: i64,
    pub total_frames: i64,
}

/// Remote progress response
#[derive(Debug, Serialize, Deserialize)]
pub struct RemoteProgressResponse {
    pub active_jobs: Vec<RemoteJobProgress>,
}

/// Leaderboard category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaderboardCategory {
    MostJobs,
    MostSaved,
    HighestSpeed,
}

impl LeaderboardCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            LeaderboardCategory::MostJobs => "most_jobs",
            LeaderboardCategory::MostSaved => "most_saved",
            LeaderboardCategory::HighestSpeed => "highest_speed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "most_jobs" => Some(LeaderboardCategory::MostJobs),
            "most_saved" => Some(LeaderboardCategory::MostSaved),
            "highest_speed" => Some(LeaderboardCategory::HighestSpeed),
            _ => None,
        }
    }
}

/// Leaderboard entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: i64,
    pub client_name: String,
    pub value: f64,
    pub formatted_value: String,
}

/// Leaderboard response
#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub category: String,
    pub entries: Vec<LeaderboardEntry>,
}
