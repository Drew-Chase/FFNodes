use crate::clients::{AuthResponse, ClientManager, ClientRegistration};
use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::jwt;
use actix_web::{web, HttpResponse};
use log::{debug, warn};
use std::sync::Arc;

/// Handshake endpoint - validates server GUID and registers client
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
        auth_token: token,
        ffmpeg_template: config.ffmpeg_template.clone(),
    };

    Ok(HttpResponse::Ok().json(response))
}
