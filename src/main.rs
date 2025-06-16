use env_logger;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

mod errors;
mod firecrawl_service;
mod models;
mod provider;
mod routes;

use crate::models::{OEmbedRequest, OEmbedResponse};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "message_type")]
enum Message {
    Request(OEmbedRequest),
    Response(OEmbedResponse),
    Error { message: String },
}

const SOCKET_PATH: &str = "/tmp/oembed_service.sock";

async fn handle_client(stream: UnixStream) -> Result<(), Box<dyn std::error::Error>> {
    info!("New client connected");
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Read the request line
    reader.read_line(&mut line).await?;

    // Parse the JSON request
    let request: Message = match serde_json::from_str(&line) {
        Ok(Message::Request(req)) => Message::Request(req),
        Ok(_) => {
            let error = Message::Error {
                message: "Invalid request".to_string(),
            };
            let response = serde_json::to_string(&error)? + "\n";
            writer.write_all(response.as_bytes()).await?;
            return Ok(());
        }
        Err(e) => {
            let error = Message::Error {
                message: format!("Error parsing request: {}", e),
            };
            let response = serde_json::to_string(&error)? + "\n";
            writer.write_all(response.as_bytes()).await?;
            return Ok(());
        }
    };

    info!("Received request: {:?}", request);

    let provider = provider::Provider::new();

    // Handle the request
    let response = if let Message::Request(req) = request {
        match provider.get_oembed(req).await {
            Ok(Some(res)) => Message::Response(res),
            Ok(None) => Message::Error {
                message: "No oEmbed data found".to_string(),
            },
            Err(e) => Message::Error {
                message: format!("Error processing request: {}", e),
            },
        }
    } else {
        Message::Error {
            message: "Invalid request".to_string(),
        }
    };

    // Send the response
    let response_str = serde_json::to_string(&response)? + "\n";
    writer.write_all(response_str.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let _ = std::fs::remove_file(SOCKET_PATH);

    let listener = UnixListener::bind(SOCKET_PATH)?;
    info!("OEmbed service listening on {:?}", SOCKET_PATH);

    // Set appropriate permissions for the socket
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o666);
        std::fs::set_permissions(SOCKET_PATH, permissions)?;
    }

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream).await {
                        error!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Error accepting connection: {}", e);
            }
        }
    }
}
