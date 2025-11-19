use actix_web::error::HttpError;
use actix_web::http::header::ToStrError;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};
use anyhow::anyhow;
use serde_json::json;
use std::path::Path;

/// Custom error types for handling various error scenarios in the application
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // Represents unspecified internal errors
    #[allow(dead_code)]
    #[error(
        "an unspecified internal error occurred: * `lib.rs` sibling file text :*:
```rust
pub mod asset_endpoint;
pub mod data_database_connection;
pub mod http_error;

```"
    )]
    InternalError(anyhow::Error),

    // Generic error type for miscellaneous errors
    #[allow(dead_code)]
    #[error(transparent)]
    Other(anyhow::Error),

    // General application error wrapper
    #[error("an error has occurred: {0:?}")]
    Anyhow(anyhow::Error),

    // Specific error for header parsing failures
    #[error("unable to parse headers: {0:?}")]
    HeaderParse(ToStrError),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::InternalError(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();

        // Extract only the error message without the backtrace
        let full_error = self.to_string();
        let error_message = match full_error.split("\n\nStack backtrace:").next() {
            Some(message) => message.trim(),
            None => &full_error, // Fallback to the full error if split fails
        };

        // For Anyhow errors, clean up the message
        let error_message = match self {
            Error::Anyhow(_) => {
                if let Some(msg) = error_message.strip_prefix("an error has occurred: ") {
                    msg
                } else {
                    error_message
                }
            }
            _ => error_message,
        };

        #[cfg(debug_assertions)]
        {
            // Capture backtrace
            let backtrace = std::backtrace::Backtrace::capture();
            let backtrace_str = backtrace.to_string();

            // Parse backtrace into a structured format
            let frames = parse_backtrace(&backtrace_str);

            return HttpResponse::build(status_code)
                .content_type("application/json")
                .json(json!({
                    "message": error_message,
                    "status": status_code.as_u16(),
                    "stacktrace": frames
                }));
        }

        #[cfg(not(debug_assertions))]
        {
            // For production - no stacktrace
            HttpResponse::build(status_code)
                .content_type("application/json")
                .json(json!({
                    "message": error_message,
                    "status": status_code.as_u16()
                }))
        }
    }
}

/// Conversion from anyhow::Error to custom Error type
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Anyhow(err)
    }
}

/// Conversion from ToStrError to custom Error type
impl From<ToStrError> for Error {
    fn from(err: ToStrError) -> Self {
        Error::HeaderParse(err)
    }
}

/// Conversion from std::io::Error to custom Error type
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Anyhow(anyhow::Error::new(err))
    }
}

/// Conversion from sqlx::Error to custom Error type
impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Error::Anyhow(anyhow::Error::new(err))
    }
}

/// Conversion from HttpError to custom Error type
impl From<HttpError> for Error {
    fn from(err: HttpError) -> Self {
        Error::Anyhow(anyhow::Error::new(err))
    }
}

/// Conversion from HttpResponse to custom Error type
impl From<HttpResponse> for Error {
    fn from(err: HttpResponse) -> Self {
        Error::Anyhow(anyhow!(
            "HTTP response error: {}",
            err.status().canonical_reason().unwrap_or("")
        ))
    }
}

// Type alias for Result using custom Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Helper to parse the backtrace into structured JSON data
fn parse_backtrace(backtrace_str: &str) -> Vec<serde_json::Value> {
    let lines: Vec<&str> = backtrace_str.lines().collect();
    let mut idx = 0;
    let mut frames = Vec::new();

    while idx < lines.len() {
        let line = lines[idx].trim_start();

        // Match lines that start with frame number ("0:", "1:", ...)
        if let Some(colon_pos) = line.find(':') {
            let (frame_number, rest) = line.split_at(colon_pos);
            if frame_number.trim().chars().all(char::is_numeric) {
                let function_name = rest[1..].trim();

                // Get next line for location information
                idx += 1;
                if idx < lines.len() {
                    let location_line = lines[idx].trim().strip_prefix("at ").unwrap_or("");

                    let (path_only, line_number) = extract_line_number(location_line);

                    let absolute_path = absolute_path(path_only);

                    frames.push(json!({
                        "file": absolute_path,
                        "function": function_name,
                        "line": line_number,
                    }));
                }
            }
        }
        idx += 1;
    }

    frames
}

/// Extract file path and line number from string
fn extract_line_number(location: &str) -> (String, i32) {
    if let Some((path, line_number_str)) = location.rsplit_once(':') {
        if let Ok(line_number) = line_number_str.parse::<i32>() {
            return (path.to_string(), line_number);
        }
    }
    (location.to_string(), -1)
}

/// Convert path to absolute path
fn absolute_path(path: String) -> String {
    if path.starts_with(".") {
        // Handle relative path starting from current directory
        std::env::current_dir()
            .map(|p| p.join(path).canonicalize().unwrap_or_default())
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    } else {
        // Path is already absolute; just clean it up
        Path::new(&path).to_path_buf().to_string_lossy().to_string()
    }
}