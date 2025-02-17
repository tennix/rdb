mod commands;
mod config;
mod protocol;
mod storage;

use crate::commands::handle_command;
use crate::config::{load_config, Config};
use crate::protocol::{parse_resp, RespError};
use crate::storage::{Db, Storage};
use bytes::BytesMut;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

use tokio::sync::Semaphore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();

    // Load configuration
    let config = load_config().unwrap_or_default();
    info!("Loaded configuration: {:?}", config);

    // Create a new database and load existing data if persistence is enabled
    let mut storage = Storage::new(config.storage.clone());
    if let Err(e) = storage.load_from_disk() {
        error!("Failed to load data from disk: {}", e);
    }
    let db: Db = Arc::new(Mutex::new(storage));
    info!("Initialized database");

    // Create connection limiter
    let connection_limit = Arc::new(Semaphore::new(config.server.max_connections));
    info!("Connection limit set to {}", config.server.max_connections);

    // Bind to configured address
    let listener = TcpListener::bind(config.server.listen_addr).await?;
    info!("Server listening on {}", config.server.listen_addr);

    loop {
        // Wait for a connection slot to become available
        let permit = connection_limit.clone().acquire_owned().await?;
        let (socket, addr) = listener.accept().await?;
        info!("New connection from {}", addr);

        let db = db.clone();

        // Handle each client in a separate task
        let config = config.clone();
        tokio::spawn(async move {
            // The permit is automatically released when dropped
            let _permit = permit;

            if let Err(e) = process_client(socket, db, &config).await {
                error!("Error processing client: {}", e);
            }
        });
    }
}

use std::time::Duration;
use tokio::time::timeout;

const CLIENT_TIMEOUT: Duration = Duration::from_secs(60);

async fn process_client(
    socket: TcpStream,
    db: Db,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = BytesMut::with_capacity(config.server.buffer_size);
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    loop {
        // Read command from client with timeout
        match timeout(CLIENT_TIMEOUT, reader.read_buf(&mut buffer)).await {
            Ok(Ok(0)) => return Ok(()), // Client disconnected
            Ok(Ok(_)) => {
                let command = String::from_utf8_lossy(&buffer);
                debug!("Received raw input: {}", command.trim());

                // Parse RESP protocol
                match parse_resp(command.as_ref()) {
                    Ok((_value, _)) => {
                        let resp = handle_command(&command, &db).await;
                        let response = resp.serialize();
                        debug!("Sending response: {}", response.trim());
                        writer.write_all(response.as_bytes()).await?;
                        writer.flush().await?;
                    }
                    Err(RespError::Incomplete) => continue, // Need more data
                    Err(e) => {
                        let err = format!("-ERR Protocol error: {}\r\n", e);
                        writer.write_all(err.as_bytes()).await?;
                        writer.flush().await?;
                    }
                }

                buffer.clear();
            }
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {
                return Err("Client timeout".into());
            }
        }
    }
}

