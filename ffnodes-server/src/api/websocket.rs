use crate::clients::ClientManager;
use crate::jobs::JobQueue;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message as WsMessage;
use futures::StreamExt;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// WebSocket event types
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsEvent {
    JobAssigned {
        job_id: String,
        client_id: String,
        media_file: String,
    },
    Progress {
        job_id: String,
        client_id: String,
        frame: i64,
        fps: f64,
        speed: String,
    },
    JobCompleted {
        job_id: String,
        client_id: String,
    },
    JobFailed {
        job_id: String,
        client_id: String,
        error: String,
    },
    ClientConnected {
        client_id: String,
        display_name: String,
    },
    ClientDisconnected {
        client_id: String,
    },
}

/// WebSocket endpoint for progress updates
pub async fn ws_progress(
    req: HttpRequest,
    stream: web::Payload,
    _job_queue: web::Data<Arc<JobQueue>>,
    _client_manager: web::Data<Arc<ClientManager>>,
) -> Result<HttpResponse, actix_web::Error> {
    debug!("WebSocket connection request");

    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn task to handle WebSocket messages
    actix_web::rt::spawn(async move {
        let _ = handle_ws_session(session, msg_stream).await;
    });

    Ok(response)
}

async fn handle_ws_session(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
) -> Result<(), actix_web::Error> {
    // Handle incoming messages
    while let Some(Ok(msg)) = msg_stream.next().await {
        match msg {
            WsMessage::Ping(bytes) => {
                let _ = session.pong(&bytes).await;
            }
            WsMessage::Text(text) => {
                debug!("Received: {}", text);
            }
            WsMessage::Close(reason) => {
                debug!("Close: {:?}", reason);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Broadcast event to all WebSocket connections
pub async fn _broadcast_event(_event: WsEvent) {
    // TODO: Implement broadcast mechanism
    // This would require maintaining a registry of active WebSocket connections
}
