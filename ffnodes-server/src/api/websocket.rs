use crate::clients::ClientManager;
use crate::jobs::JobQueue;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_ws::Message as WsMessage;
use futures::StreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use tokio::sync::mpsc;

/// Type alias for WebSocket message sender
type WsSender = mpsc::UnboundedSender<WsEvent>;

/// WebSocket connection registry
/// Maps client_id -> list of WebSocket senders for that client
pub type WsRegistry = Arc<RwLock<HashMap<String, Vec<WsSender>>>>;

/// Create a new WebSocket registry
pub fn create_ws_registry() -> WsRegistry {
    Arc::new(RwLock::new(HashMap::new()))
}

/// WebSocket event types
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    query: web::Query<HashMap<String, String>>,
    registry: web::Data<WsRegistry>,
) -> Result<HttpResponse, actix_web::Error> {
    debug!("WebSocket connection request");

    // Get client_id from query params
    let client_id = query.get("client_id").cloned().unwrap_or_else(|| "unknown".to_string());
    debug!("WebSocket connection from client: {}", client_id);

    let (response, session, msg_stream) = actix_ws::handle(&req, stream)?;

    // Create channel for receiving events
    let (tx, rx) = mpsc::unbounded_channel();

    // Register this connection
    {
        let mut registry = registry.write().await;
        registry.entry(client_id.clone()).or_insert_with(Vec::new).push(tx);
        debug!("Registered WebSocket connection for client: {}", client_id);
    }

    // Spawn task to handle WebSocket messages
    let registry_clone = registry.get_ref().clone();
    actix_web::rt::spawn(async move {
        let _ = handle_ws_session(session, msg_stream, rx, client_id, registry_clone).await;
    });

    Ok(response)
}

async fn handle_ws_session(
    mut session: actix_ws::Session,
    mut msg_stream: actix_ws::MessageStream,
    mut event_rx: mpsc::UnboundedReceiver<WsEvent>,
    client_id: String,
    registry: WsRegistry,
) -> Result<(), actix_web::Error> {
    // Handle incoming messages and outgoing events
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages from client
            Some(Ok(msg)) = msg_stream.next() => {
                match msg {
                    WsMessage::Ping(bytes) => {
                        let _ = session.pong(&bytes).await;
                    }
                    WsMessage::Text(text) => {
                        debug!("Received from {}: {}", client_id, text);
                    }
                    WsMessage::Close(reason) => {
                        debug!("Client {} closed connection: {:?}", client_id, reason);
                        break;
                    }
                    _ => {}
                }
            }
            // Handle outgoing events to send to client
            Some(event) = event_rx.recv() => {
                // Serialize event to JSON
                match serde_json::to_string(&event) {
                    Ok(json) => {
                        if let Err(e) = session.text(json).await {
                            debug!("Failed to send event to {}: {}", client_id, e);
                            break;
                        }
                    }
                    Err(e) => {
                        debug!("Failed to serialize event: {}", e);
                    }
                }
            }
            else => {
                // Both streams closed
                break;
            }
        }
    }

    // Cleanup: remove this connection from registry
    {
        let mut registry = registry.write().await;
        if let Some(senders) = registry.get_mut(&client_id) {
            senders.retain(|sender| !sender.is_closed());
            if senders.is_empty() {
                registry.remove(&client_id);
                debug!("Removed all connections for client: {}", client_id);
            }
        }
    }

    let _ = session.close(None).await;
    debug!("WebSocket session closed for client: {}", client_id);

    Ok(())
}

/// Broadcast event to all WebSocket connections
pub async fn broadcast_event(registry: &WsRegistry, event: WsEvent) {
    let registry = registry.read().await;
    let mut dead_senders = Vec::new();

    // Send event to all registered connections
    for (client_id, senders) in registry.iter() {
        for (idx, sender) in senders.iter().enumerate() {
            if let Err(_) = sender.send(event.clone()) {
                debug!("Failed to send event to client {}, sender {}", client_id, idx);
                dead_senders.push((client_id.clone(), idx));
            }
        }
    }

    if !dead_senders.is_empty() {
        debug!("Detected {} dead WebSocket connections", dead_senders.len());
    }
}
