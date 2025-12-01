use actix_web::{get, HttpResponse, Responder};
use serde_json::json;
use crate::http_error::Result;
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[get("ping")]
pub async fn ping() -> Result<impl Responder> {
    Ok(HttpResponse::Ok().json(json!({
        "version": VERSION,
    })))
}

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("")
            .service(ping)
            .default_service(actix_web::web::to(|| async {
            HttpResponse::NotFound().json(json!({
                "error": "API endpoint not found".to_string(),
            }))
        })),
    );
}
