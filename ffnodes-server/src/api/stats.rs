use crate::http_error::Error as HttpError;
use crate::stats::{models::LeaderboardCategory, queries};
use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;

/// Get client history statistics
/// GET /api/stats/client/{client_id}/history
pub async fn get_client_history(
    client_id: web::Path<String>,
    pool: web::Data<SqlitePool>,
) -> Result<HttpResponse, HttpError> {
    let history = queries::get_client_history(&pool, &client_id).await?;
    Ok(HttpResponse::Ok().json(history))
}

/// Get remote users' progress
/// GET /api/stats/remote-progress?exclude_self={client_id}
pub async fn get_remote_progress(
    pool: web::Data<SqlitePool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, HttpError> {
    let exclude_client = query.get("exclude_self").map(|s| s.as_str());
    let progress = queries::get_remote_progress(&pool, exclude_client).await?;
    Ok(HttpResponse::Ok().json(progress))
}

/// Get leaderboard data
/// GET /api/stats/leaderboard?category={most_jobs|most_saved|highest_speed}
pub async fn get_leaderboard(
    pool: web::Data<SqlitePool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<HttpResponse, HttpError> {
    let category_str = query
        .get("category")
        .map(|s| s.as_str())
        .unwrap_or("most_jobs");

    let category = LeaderboardCategory::from_str(category_str)
        .unwrap_or(LeaderboardCategory::MostJobs);

    let leaderboard = queries::get_leaderboard(&pool, category).await?;
    Ok(HttpResponse::Ok().json(leaderboard))
}
