mod commands;
mod config;
mod protocol;
mod storage;

use crate::config::{Config, load_config};

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::commands::handle_command;
use crate::storage::Db;
use log::{info, debug, error};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{BufWriter, AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    // Load configuration
    let config = load_config().unwrap_or_default();
    info!("Loaded configuration: {:?}", config);
    
    // Create a new in-memory database
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    info!("Initialized in-memory database");
    
    // Bind to configured address
    let listener = TcpListener::bind(config.server.listen_addr).await?;
    info!("Server listening on {}", config.server.listen_addr);

    loop {
        let (socket, _) = listener.accept().await?;
        let db = db.clone();
        
        // Handle each client in a separate task
        tokio::spawn(async move {
            if let Err(e) = process_client(socket, db).await {
                error!("Error processing client: {}", e);
            }
        });
    }
}

async fn process_client(
    socket: TcpStream,
    db: Db,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = BytesMut::with_capacity(config.server.buffer_size);
    let mut writer = BufWriter::new(socket);

    loop {
        // Read command from client
        let n = writer.read_buf(&mut buffer).await?;
        if n == 0 {
            return Ok(());  // Client disconnected
        }

        let command = String::from_utf8_lossy(&buffer[..]);
        debug!("Received command: {}", command.trim());
        let response = handle_command(&command, &db).await;
        debug!("Sending response: {}", response.trim());
        
        // Send response back to client
        writer.write_all(response.as_bytes()).await?;
        writer.flush().await?;
        
        buffer.clear();
    }
}

