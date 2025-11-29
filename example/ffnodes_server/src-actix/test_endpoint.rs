use actix_web::{get, web, HttpResponse, Responder};
use crate::http_error::Result;
use serde_json::json;
/// Handles requests to check the server status.
///
/// This endpoint responds to GET requests with a JSON object indicating
/// that the server is running correctly. It can be used for health checks
/// or monitoring server status.
///
/// # Returns
///
/// A JSON object with a `status` field set to "ok".
#[get("/")]
async fn status() -> Result<impl Responder> {
	Ok(HttpResponse::Ok().json(json!({ "status": "ok" })))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
	cfg.service(web::scope("/").service(status));
}
