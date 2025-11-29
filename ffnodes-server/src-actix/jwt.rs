use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (client_id)
    pub exp: i64,         // Expiration time (Unix timestamp)
    pub iat: i64,         // Issued at (Unix timestamp)
}

impl Claims {
    pub fn new(client_id: String, expires_in_days: i64) -> Self {
        let now = Utc::now();
        let exp = now + Duration::days(expires_in_days);

        Self {
            sub: client_id,
            iat: now.timestamp(),
            exp: exp.timestamp(),
        }
    }
}

/// Generate a JWT token for a client
pub fn generate_token(client_id: String, secret: &str, expires_in_days: i64) -> anyhow::Result<String> {
    let claims = Claims::new(client_id, expires_in_days);
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

/// Validate and decode a JWT token
pub fn validate_token(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
