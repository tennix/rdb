mod commands;
mod protocol;
mod storage;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::commands::{Command, handle_command};
use crate::storage::Db;
use log::{info, debug, error};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{BufWriter, AsyncReadExt, AsyncWriteExt};
use bytes::BytesMut;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    // Create a new in-memory database
    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    info!("Initialized in-memory database");
    
    // Bind to localhost:6379 (default Redis port)
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    info!("Server listening on port 6379");

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
    let mut buffer = BytesMut::with_capacity(1024);
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

