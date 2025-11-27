use super::models::*;
use anyhow::Result;
use sqlx::SqlitePool;

/// Get client history with completed jobs
pub async fn get_client_history(pool: &SqlitePool, client_id: &str) -> Result<ClientHistoryResponse> {
    // Query all completed jobs for this client with necessary data
    let jobs: Vec<(String, i64, Option<i64>, i64, i64, i64, String, f64)> = sqlx::query_as(
        r#"
        SELECT
            ej.media_file_path,
            mf.scanned_size,
            ej.output_size,
            ej.started_at,
            ej.completed_at,
            mf.frames,
            ep.speed,
            ep.fps
        FROM encoding_jobs ej
        JOIN media_files mf ON ej.media_file_path = mf.path
        LEFT JOIN encoding_progress ep ON ej.id = ep.job_id
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

    for (path, scanned_size, output_size, started_at, completed_at, _frames, speed_str, _fps) in jobs {
        let duration = completed_at - started_at;
        let size_after = output_size.unwrap_or(scanned_size);
        let size_saved = scanned_size - size_after;
        let size_reduction_percent = if scanned_size > 0 {
            (size_saved as f64 / scanned_size as f64) * 100.0
        } else {
            0.0
        };

        // Parse speed (format: "2.5x")
        let average_speed = speed_str
            .trim_end_matches('x')
            .parse::<f64>()
            .unwrap_or(0.0);

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
    let jobs: Vec<(String, String, i64, i64, String, f64)> = sqlx::query_as(
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
        .map(|(client_name, path, frame, total_frames, speed_str, _fps)| {
            let filename = path.split(['/', '\\']).last().unwrap_or(&path).to_string();
            let percentage = if total_frames > 0 {
                (frame as f64 / total_frames as f64) * 100.0
            } else {
                0.0
            };
            let speed = speed_str
                .trim_end_matches('x')
                .parse::<f64>()
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
            let results: Vec<(String, String)> = sqlx::query_as(
                r#"
                SELECT c.display_name, AVG(
                    CAST(REPLACE(ep.speed, 'x', '') AS REAL)
                ) as avg_speed
                FROM encoding_jobs ej
                JOIN clients c ON ej.assigned_client = c.id
                LEFT JOIN encoding_progress ep ON ej.id = ep.job_id
                WHERE ej.status = 'completed'
                AND ep.speed IS NOT NULL
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
                .filter_map(|(idx, (name, speed_str))| {
                    speed_str.parse::<f64>().ok().map(|speed| LeaderboardEntry {
                        rank: (idx + 1) as i64,
                        client_name: name,
                        value: speed,
                        formatted_value: format!("{:.2}x", speed),
                    })
                })
                .collect()
        }
    };

    Ok(LeaderboardResponse {
        category: category.as_str().to_string(),
        entries,
    })
}
