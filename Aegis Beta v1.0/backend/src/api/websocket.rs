use futures::{StreamExt, SinkExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::vm::manager::VMManager;

pub async fn start_websocket_server(vm_manager: Arc<VMManager>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(&addr).await?;
    log::info!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let vm_manager_clone = vm_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, vm_manager_clone).await {
                log::error!("WebSocket error: {}", e);
            }
        });
    }

    Ok(())
}

async fn handle_connection(stream: TcpStream, vm_manager: Arc<VMManager>) -> Result<(), Box<dyn std::error::Error>> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    // Handle incoming messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse command
                if let Ok(cmd) = serde_json::from_str::<WebSocketCommand>(&text) {
                    match cmd {
                        WebSocketCommand::Subscribe { vm_id } => {
                            // Subscribe to VM updates
                            let status = vm_manager.get_vm_status(&vm_id).await;
                            if let Some(status) = status {
                                let response = WebSocketResponse::VmStatus { status };
                                let json = serde_json::to_string(&response).unwrap();
                                write.send(Message::Text(json)).await?;
                            }
                        }
                        WebSocketCommand::ConsoleInput { vm_id, input } => {
                            // Send input to VM console
                            if let Err(e) = vm_manager.send_console_input(&vm_id, &input).await {
                                let error = WebSocketResponse::Error { message: e.to_string() };
                                let json = serde_json::to_string(&error).unwrap();
                                write.send(Message::Text(json)).await?;
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                break;
            }
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum WebSocketCommand {
    Subscribe { vm_id: String },
    ConsoleInput { vm_id: String, input: String },
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
enum WebSocketResponse {
    VmStatus { status: crate::vm::config::VMStatus },
    ConsoleOutput { output: String },
    Error { message: String },
}