// API Response Types (matching backend Rust structs)

export interface OverallSystemStats {
  total_media_files: number;
  processed_files: number;
  pending_files: number;
  total_storage_bytes: number;
  total_saved_bytes: number;
  total_processing_time_seconds: number;
  average_encoding_speed: number;
  total_jobs_completed: number;
  total_jobs_failed: number;
}

export interface SystemStatus {
  total_media_files: number;
  pending_jobs: number;
  active_jobs: number;
  connected_clients: number;
}

export interface ClientStatus {
  id: string;
  display_name: string;
  computer_name: string;
  connected_at: number;
  last_heartbeat: number;
  disconnected_at: number | null;
}

export interface EncodingJob {
  id: string;
  media_file_path: string;
  status: 'pending' | 'assigned' | 'in_progress' | 'completed' | 'failed';
  priority: number;
  assigned_client: string | null;
  assigned_at: number | null;
  started_at: number | null;
  completed_at: number | null;
  error_message: string | null;
  output_path: string | null;
  output_size: number | null;
  output_bitrate: number | null;
  average_speed: number | null;
  current_phase: string | null; // "downloading", "encoding", "uploading"
  created_at: number;
}

export interface JobHistoryEntry {
  filename: string;
  average_speed: number;
  duration_seconds: number;
  size_before: number;
  size_after: number;
  size_saved: number;
  size_reduction_percent: number;
  completed_at: number;
}

export interface OverallStats {
  average_speed: number;
  average_duration_seconds: number;
  average_size_reduction_percent: number;
  total_jobs: number;
  total_size_saved: number;
}

export interface ClientHistoryResponse {
  jobs: JobHistoryEntry[];
  overall: OverallStats;
}

export interface RemoteJobProgress {
  client_name: string;
  filename: string;
  percentage: number;
  speed: number;
  frame: number;
  total_frames: number;
}

export interface RemoteProgressResponse {
  active_jobs: RemoteJobProgress[];
}

export type LeaderboardCategory = 'most_jobs' | 'most_saved' | 'highest_speed';

export interface LeaderboardEntry {
  rank: number;
  client_name: string;
  value: number;
  formatted_value: string;
}

export interface LeaderboardResponse {
  category: string;
  entries: LeaderboardEntry[];
}

export interface ProgressUpdate {
  frame: number;
  fps: number;
  bitrate: string;
  speed: string;
}

// Dashboard-specific types

export interface DashboardEvent {
  event: 'job_started' | 'job_completed' | 'job_failed' | 'client_connected' | 'client_disconnected' | 'progress_update';
  job_id?: string;
  client_id?: string;
  filename?: string;
  duration?: number;
  saved_bytes?: number;
  error?: string;
  display_name?: string;
  percentage?: number;
  speed?: string;
}
