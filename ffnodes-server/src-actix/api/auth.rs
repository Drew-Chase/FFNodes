use crate::clients::{AuthResponse, ClientManager, ClientRegistration};
use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::jwt;
use actix_web::{post, web, HttpResponse};
use log::{debug, warn};
use serde_json::json;
use std::sync::Arc;

/// Handshake endpoint - validates server GUID and registers client
#[post("/handshake")]
pub async fn handshake(
    registration: web::Json<ClientRegistration>,
    config: web::Data<Arc<Configuration>>,
    client_manager: web::Data<Arc<ClientManager>>,
) -> Result<HttpResponse, Error> {
    debug!("Handshake request from: {}", registration.display_name);

    // Validate server GUID
    if registration.server_guid != config.server_guid {
        warn!("Invalid server GUID from: {}", registration.display_name);
        return Err(Error::unauthorized("Invalid server GUID"));
    }

    // Register client
    let client = client_manager
        .register_client(
            registration.display_name.clone(),
            registration.computer_name.clone(),
            Some(registration.machine_id.clone()),
        )
        .await
        .map_err(|e| {
            warn!("Failed to register client: {:#}", e);
            Error::internal_server_error("Failed to register client")
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
