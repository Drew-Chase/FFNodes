use super::models::*;
use anyhow::Result;
use sqlx::SqlitePool;

/// Get client history with completed jobs
pub async fn get_client_history(pool: &SqlitePool, client_id: &str) -> Result<ClientHistoryResponse> {
    // Query all completed jobs for this client with necessary data
    let jobs: Vec<(String, i64, Option<i64>, i64, i64, Option<f64>)> = sqlx::query_as(
        r#"
        SELECT
            ej.media_file_path,
            mf.scanned_size,
            ej.output_size,
            ej.started_at,
            ej.completed_at,
            ej.average_speed
        FROM encoding_jobs ej
        JOIN media_files mf ON ej.media_file_path = mf.path
        WHERE ej.assigned_client = ?
        AND ej.status = 'completed'
        AND ej.output_size IS NOT NULL
        ORDER BY ej.completed_at DESC
        LIMIT 100
        "#,
    )
    .bind(client_id)
    .fetch_all(pool)
    .await?;

    let mut history_entries = Vec::new();
    let mut total_speed = 0.0;
    let mut total_duration = 0i64;
    let mut total_size_before = 0i64;
    let mut total_size_after = 0i64;
    let mut speed_count = 0;

    for (path, scanned_size, output_size, started_at, completed_at, speed_opt) in jobs {
        let duration = completed_at - started_at;
        let size_after = output_size.unwrap_or(scanned_size);
        let size_saved = scanned_size - size_after;
        let size_reduction_percent = if scanned_size > 0 {
            (size_saved as f64 / scanned_size as f64) * 100.0
        } else {
            0.0
        };

        // Use average_speed directly from encoding_jobs table
        let average_speed = speed_opt.unwrap_or(0.0);

        // Extract filename from path
        let filename = path.split(['/', '\\']).last().unwrap_or(&path).to_string();

        history_entries.push(JobHistoryEntry {
            filename,
            average_speed,
            duration_seconds: duration,
            size_before: scanned_size,
            size_after,
            size_saved,
            size_reduction_percent,
            completed_at,
        });

        total_speed += average_speed;
        total_duration += duration;
        total_size_before += scanned_size;
        total_size_after += size_after;
        if average_speed > 0.0 {
            speed_count += 1;
        }
    }

    let job_count = history_entries.len() as i64;
    let overall = if job_count > 0 {
        let avg_speed = if speed_count > 0 {
            total_speed / speed_count as f64
        } else {
            0.0
        };
        let total_saved = total_size_before - total_size_after;
        let avg_reduction = if total_size_before > 0 {
            (total_saved as f64 / total_size_before as f64) * 100.0
        } else {
            0.0
        };

        OverallStats {
            average_speed: avg_speed,
            average_duration_seconds: total_duration as f64 / job_count as f64,
            average_size_reduction_percent: avg_reduction,
            total_jobs: job_count,
            total_size_saved: total_saved,
        }
    } else {
        OverallStats {
            average_speed: 0.0,
            average_duration_seconds: 0.0,
            average_size_reduction_percent: 0.0,
            total_jobs: 0,
            total_size_saved: 0,
        }
    };

    Ok(ClientHistoryResponse {
        jobs: history_entries,
        overall,
    })
}

/// Get remote user progress for all active jobs
pub async fn get_remote_progress(pool: &SqlitePool, exclude_client_id: Option<&str>) -> Result<RemoteProgressResponse> {
    // Query all in-progress jobs with their progress and client info
    let jobs: Vec<(String, String, Option<i64>, i64, Option<String>, Option<f64>)> = sqlx::query_as(
        r#"
        SELECT
            c.display_name,
            ej.media_file_path,
            ep.frame,
            mf.frames,
            ep.speed,
            ep.fps
        FROM encoding_jobs ej
        JOIN clients c ON ej.assigned_client = c.id
        JOIN media_files mf ON ej.media_file_path = mf.path
        LEFT JOIN encoding_progress ep ON ej.id = ep.job_id
        WHERE ej.status = 'in_progress'
        AND (? IS NULL OR ej.assigned_client != ?)
        ORDER BY ep.updated_at DESC
        "#,
    )
    .bind(exclude_client_id)
    .bind(exclude_client_id)
    .fetch_all(pool)
    .await?;

    let active_jobs: Vec<RemoteJobProgress> = jobs
        .into_iter()
        .map(|(client_name, path, frame_opt, total_frames, speed_opt, _fps)| {
            let filename = path.split(['/', '\\']).last().unwrap_or(&path).to_string();
            let frame = frame_opt.unwrap_or(0);
            let percentage = if total_frames > 0 {
                (frame as f64 / total_frames as f64) * 100.0
            } else {
                0.0
            };
            let speed = speed_opt
                .as_ref()
                .and_then(|s| s.trim_end_matches('x').parse::<f64>().ok())
                .unwrap_or(0.0);

            RemoteJobProgress {
                client_name,
                filename,
                percentage,
                speed,
                frame,
                total_frames,
            }
        })
        .collect();

    Ok(RemoteProgressResponse { active_jobs })
}

/// Get leaderboard data by category
pub async fn get_leaderboard(
    pool: &SqlitePool,
    category: LeaderboardCategory,
) -> Result<LeaderboardResponse> {
    let entries = match category {
        LeaderboardCategory::MostJobs => {
            // Count completed jobs per client
            let results: Vec<(String, i64)> = sqlx::query_as(
                r#"
                SELECT c.display_name, COUNT(*) as job_count
                FROM encoding_jobs ej
                JOIN clients c ON ej.assigned_client = c.id
                WHERE ej.status = 'completed'
                GROUP BY ej.assigned_client, c.display_name
                ORDER BY job_count DESC
                LIMIT 10
                "#,
            )
            .fetch_all(pool)
            .await?;

            results
                .into_iter()
                .enumerate()
                .map(|(idx, (name, count))| LeaderboardEntry {
                    rank: (idx + 1) as i64,
                    client_name: name,
                    value: count as f64,
                    formatted_value: format!("{} jobs", count),
                })
                .collect()
        }
        LeaderboardCategory::MostSaved => {
            // Sum of size savings per client
            let results: Vec<(String, i64)> = sqlx::query_as(
                r#"
                SELECT c.display_name, SUM(mf.scanned_size - COALESCE(ej.output_size, mf.scanned_size)) as total_saved
                FROM encoding_jobs ej
                JOIN clients c ON ej.assigned_client = c.id
                JOIN media_files mf ON ej.media_file_path = mf.path
                WHERE ej.status = 'completed'
                AND ej.output_size IS NOT NULL
                GROUP BY ej.assigned_client, c.display_name
                ORDER BY total_saved DESC
                LIMIT 10
                "#,
            )
            .fetch_all(pool)
            .await?;

            results
                .into_iter()
                .enumerate()
                .map(|(idx, (name, saved))| {
                    let gb = saved as f64 / 1_073_741_824.0; // Convert to GB
                    LeaderboardEntry {
                        rank: (idx + 1) as i64,
                        client_name: name,
                        value: saved as f64,
                        formatted_value: format!("{:.2} GB", gb),
                    }
                })
                .collect()
        }
        LeaderboardCategory::HighestSpeed => {
            // Average encoding speed per client
            let results: Vec<(String, f64)> = sqlx::query_as(
                r#"
                SELECT c.display_name, AVG(ej.average_speed) as avg_speed
                FROM encoding_jobs ej
                JOIN clients c ON ej.assigned_client = c.id
                WHERE ej.status = 'completed'
                AND ej.average_speed IS NOT NULL
                GROUP BY ej.assigned_client, c.display_name
                HAVING COUNT(*) >= 3
                ORDER BY avg_speed DESC
                LIMIT 10
                "#,
            )
            .fetch_all(pool)
            .await?;

            results
                .into_iter()
                .enumerate()
                .map(|(idx, (name, speed))| LeaderboardEntry {
                    rank: (idx + 1) as i64,
                    client_name: name,
                    value: speed,
                    formatted_value: format!("{:.2}x", speed),
                })
                .collect()
        }
    };

    Ok(LeaderboardResponse {
        category: category.as_str().to_string(),
        entries,
    })
}

/// Get overall system statistics for dashboard
pub async fn get_overall_stats(pool: &SqlitePool) -> Result<OverallSystemStats> {
    // Count total media files
    let total_media_files: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM media_files")
        .fetch_one(pool)
        .await?;

    // Count processed files
    let processed_files: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM media_files WHERE processed = 1")
        .fetch_one(pool)
        .await?;

    // Calculate pending files
    let pending_files = total_media_files.0 - processed_files.0;

    // Sum storage (original scanned size vs current size)
    let storage_stats: (Option<i64>, Option<i64>) = sqlx::query_as(
        r#"
        SELECT
            SUM(scanned_size) as total_scanned,
            SUM(COALESCE(size, scanned_size)) as total_current
        FROM media_files
        WHERE processed = 1
        "#
    )
    .fetch_one(pool)
    .await?;

    let total_storage_bytes = storage_stats.0.unwrap_or(0);
    let current_storage = storage_stats.1.unwrap_or(0);
    let total_saved_bytes = total_storage_bytes - current_storage;

    // Total processing time (sum of completed job durations)
    let processing_time: (Option<i64>,) = sqlx::query_as(
        r#"
        SELECT SUM(completed_at - started_at)
        FROM encoding_jobs
        WHERE status = 'completed'
        "#
    )
    .fetch_one(pool)
    .await?;

    let total_processing_time_seconds = processing_time.0.unwrap_or(0);

    // Average encoding speed
    let avg_speed: (Option<f64>,) = sqlx::query_as(
        r#"
        SELECT AVG(average_speed)
        FROM encoding_jobs
        WHERE status = 'completed'
        AND average_speed IS NOT NULL
        "#
    )
    .fetch_one(pool)
    .await?;

    let average_encoding_speed = avg_speed.0.unwrap_or(0.0);

    // Job counts
    let completed_jobs: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM encoding_jobs WHERE status = 'completed'"
    )
    .fetch_one(pool)
    .await?;

    let failed_jobs: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM encoding_jobs WHERE status = 'failed'"
    )
    .fetch_one(pool)
    .await?;

    Ok(OverallSystemStats {
        total_media_files: total_media_files.0,
        processed_files: processed_files.0,
        pending_files,
        total_storage_bytes,
        total_saved_bytes,
        total_processing_time_seconds,
        average_encoding_speed,
        total_jobs_completed: completed_jobs.0,
        total_jobs_failed: failed_jobs.0,
    })
}
