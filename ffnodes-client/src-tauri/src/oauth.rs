use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::command;
use tauri_plugin_oauth::{OauthConfig, start_with_config};

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthFlowParams {
    pub provider: String,
    pub client_id: String,
    pub scope: String,
    pub redirect_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u64,
    pub token_type: String,
    pub id_token: Option<String>,
}

/// Start OAuth flow for the specified provider
/// This will open the authorization URL in the default browser and wait for the callback
#[command]
pub async fn start_oauth_flow(params: OAuthFlowParams) -> Result<OAuthResponse, String> {
    // Build authorization URL based on provider
    let port_result = start_oauth_server()?;
    let port = port_result.0;
    let received_url = port_result.1;

    let redirect_uri = format!("http://localhost:{}", port);

    let auth_url = match params.provider.as_str() {
        "google" => format!(
            "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}",
            params.client_id,
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&params.scope)
        ),
        "github" => format!(
            "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope={}",
            params.client_id,
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&params.scope)
        ),
        "microsoft" => format!(
            "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}",
            params.client_id,
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&params.scope)
        ),
        "facebook" => format!(
            "https://www.facebook.com/v18.0/dialog/oauth?client_id={}&redirect_uri={}&scope={}",
            params.client_id,
            urlencoding::encode(&redirect_uri),
            urlencoding::encode(&params.scope)
        ),
        _ => return Err(format!("Unsupported OAuth provider: {}", params.provider)),
    };

    // Open the authorization URL in the default browser
    open::that(&auth_url).map_err(|e| format!("Failed to open browser: {}", e))?;

    // Wait for the callback URL (with timeout)
    let url = tokio::time::timeout(std::time::Duration::from_secs(120), async {
        loop {
            // Check if URL has been received
            let url_option = {
                let url_guard = received_url.lock().unwrap();
                url_guard.clone()
            }; // Guard is dropped here before await

            if let Some(url) = url_option {
                return Ok::<String, String>(url);
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .map_err(|_| "OAuth flow timed out".to_string())?
    .map_err(|e| format!("Failed to receive callback: {}", e))?;

    // Parse the URL to extract the authorization code
    let parsed_url =
        url::Url::parse(&url).map_err(|e| format!("Failed to parse callback URL: {}", e))?;

    let code = parsed_url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string())
        .ok_or_else(|| "No authorization code in callback URL".to_string())?;

    // Exchange code for tokens
    exchange_code_for_tokens(&params.provider, &code, &params.client_id, &redirect_uri).await
}

/// Type alias for the OAuth server result
type OAuthServerResult = Result<(u16, Arc<Mutex<Option<String>>>), String>;

/// Start the OAuth callback server and return the port and a reference to receive the URL
fn start_oauth_server() -> OAuthServerResult {
    let received_url = Arc::new(Mutex::new(None));
    let received_url_clone = Arc::clone(&received_url);

    // Configure OAuth server
    let config = OauthConfig {
        ports: Some(vec![3000, 3001, 3002]),
        response: Some("<html><head><title>FFNodes - Authentication Successful</title></head><body><h1>Authentication Successful!</h1><p>You can close this window and return to the FFNodes app.</p></body></html>".into()),
    };

    // Start the OAuth callback server
    let port = start_with_config(config, move |url| {
        let mut received = received_url_clone.lock().unwrap();
        *received = Some(url);
    })
    .map_err(|e| format!("Failed to start OAuth server: {}", e))?;

    Ok((port, received_url))
}

/// Exchange authorization code for access token
async fn exchange_code_for_tokens(
    provider: &str,
    code: &str,
    client_id: &str,
    redirect_uri: &str,
) -> Result<OAuthResponse, String> {
    // Get token endpoint based on provider
    let token_url = match provider {
        "google" => "https://oauth2.googleapis.com/token",
        "github" => "https://github.com/login/oauth/access_token",
        "microsoft" => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        "facebook" => "https://graph.facebook.com/v18.0/oauth/access_token",
        _ => return Err(format!("Unsupported provider: {}", provider)),
    };

    // Build request body
    // NOTE: In production, the client_secret should be stored securely
    // and this exchange should happen on your backend server
    let client_secret =
        std::env::var(format!("{}_CLIENT_SECRET", provider.to_uppercase())).unwrap_or_default();

    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("client_id", client_id),
        ("client_secret", &client_secret),
        ("redirect_uri", redirect_uri),
    ];

    // Make request to token endpoint
    let client = reqwest::Client::new();
    let response = client
        .post(token_url)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token exchange request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {}", error_text));
    }

    let token_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    // Parse response
    Ok(OAuthResponse {
        access_token: token_data["access_token"]
            .as_str()
            .ok_or("Missing access_token")?
            .to_string(),
        refresh_token: token_data["refresh_token"].as_str().map(String::from),
        expires_in: token_data["expires_in"].as_u64().unwrap_or(3600),
        token_type: token_data["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string(),
        id_token: token_data["id_token"].as_str().map(String::from),
    })
}

/// Refresh an expired access token using a refresh token
#[command]
pub async fn refresh_oauth_token(
    provider: String,
    refresh_token: String,
) -> Result<OAuthResponse, String> {
    // Get token endpoint based on provider
    let token_url = match provider.as_str() {
        "google" => "https://oauth2.googleapis.com/token",
        "github" => return Err("GitHub tokens do not expire and cannot be refreshed".to_string()),
        "microsoft" => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
        "facebook" => "https://graph.facebook.com/v18.0/oauth/access_token",
        _ => return Err(format!("Unsupported provider: {}", provider)),
    };

    let client_id = std::env::var(format!("{}_CLIENT_ID", provider.to_uppercase()))
        .map_err(|_| format!("{} client ID not configured", provider))?;

    let client_secret = std::env::var(format!("{}_CLIENT_SECRET", provider.to_uppercase()))
        .map_err(|_| format!("{} client secret not configured", provider))?;

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", &refresh_token),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
    ];

    let client = reqwest::Client::new();
    let response = client
        .post(token_url)
        .header("Accept", "application/json")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("Token refresh failed: {}", error_text));
    }

    let token_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    Ok(OAuthResponse {
        access_token: token_data["access_token"]
            .as_str()
            .ok_or("Missing access_token")?
            .to_string(),
        refresh_token: token_data["refresh_token"]
            .as_str()
            .map(String::from)
            .or(Some(refresh_token)),
        expires_in: token_data["expires_in"].as_u64().unwrap_or(3600),
        token_type: token_data["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string(),
        id_token: token_data["id_token"].as_str().map(String::from),
    })
}
