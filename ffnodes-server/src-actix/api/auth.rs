use crate::clients::{AuthResponse, ClientRegistration};
use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::job_actor::{ActorError, JobActorHandle};
use crate::jwt;
use actix_web::{post, web, HttpResponse};
use log::{debug, error, warn};
use serde_json::json;
use std::sync::Arc;

/// Convert actor error to HTTP error
fn map_actor_error(err: ActorError) -> Error {
    match err {
        ActorError::NotInitialized => {
            Error::service_unavailable("Server is still initializing, please try again")
        }
        ActorError::DatabaseError(msg) => {
            error!("Database error: {}", msg);
            Error::internal_server_error(&msg)
        }
        ActorError::NotFound => {
            Error::not_found("Resource not found")
        }
        ActorError::InvalidState(msg) => {
            error!("Invalid state: {}", msg);
            Error::internal_server_error(&msg)
        }
    }
}

/// Handshake endpoint - validates server GUID and registers client
#[post("/handshake")]
pub async fn handshake(
    registration: web::Json<ClientRegistration>,
    config: web::Data<Arc<Configuration>>,
    actor: web::Data<JobActorHandle>,
) -> Result<HttpResponse, Error> {
    debug!("Handshake request from: {}", registration.display_name);

    // Validate server GUID
    if registration.server_guid != config.server_guid {
        warn!("Invalid server GUID from: {}", registration.display_name);
        return Err(Error::unauthorized("Invalid server GUID"));
    }

    // Register client
    let client = actor
        .register_client(
            registration.display_name.clone(),
            registration.computer_name.clone(),
            Some(registration.machine_id.clone()),
        )
        .await
        .map_err(|e| {
            warn!("Failed to register client: {:#}", e);
            map_actor_error(e)
        })?;

    debug!("Client registered: {} ({})", client.display_name, client.id);

    // Generate JWT token (expires in 90 days)
    let token = jwt::generate_token(client.id.clone(), &config.jwt_secret, 90)
        .map_err(|e| {
            warn!("Failed to generate JWT token: {:#}", e);
            Error::internal_server_error("Failed to generate authentication token")
        })?;

    let response = AuthResponse {
        client_id: client.id.clone(),
        auth_token: token,
        ffmpeg_template: config.ffmpeg_template.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(handshake)
            .default_service(web::to(|| async {
                HttpResponse::NotFound().json(json!({
                    "error": "API endpoint not found".to_string(),
                }))
            })),
    );
}
