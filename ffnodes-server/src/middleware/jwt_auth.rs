use crate::configuration::Configuration;
use crate::http_error::Error;
use crate::jwt;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    HttpMessage,
};
use std::sync::Arc;
use tracing::warn;

/// JWT Authentication Middleware
///
/// Validates JWT tokens in the Authorization header.
/// Extracts client_id from token and adds it to request extensions.
pub async fn jwt_auth(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    // Get configuration from app data
    let config = req
        .app_data::<actix_web::web::Data<Arc<Configuration>>>()
        .ok_or_else(|| Error::internal_server_error("Configuration not found"))?;

    // Extract Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| Error::unauthorized("Missing Authorization header"))?;

    // Check Bearer token format
    if !auth_header.starts_with("Bearer ") {
        return Err(Error::unauthorized("Invalid Authorization header format").into());
    }

    let token = &auth_header[7..]; // Remove "Bearer " prefix

    // Validate token
    let claims = jwt::validate_token(token, &config.jwt_secret).map_err(|e| {
        warn!("JWT validation failed: {:#}", e);
        Error::unauthorized("Invalid or expired token")
    })?;

    // Add client_id to request extensions
    req.extensions_mut().insert(claims.sub.clone());

    next.call(req).await
}
