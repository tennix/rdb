use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use log::{info, debug, error};
use tokio::sync::Mutex;
use bytes::BytesMut;
use tokio::io::{BufWriter, AsyncReadExt, AsyncWriteExt};

type Db = Arc<Mutex<HashMap<String, String>>>;

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
    mut socket: TcpStream,
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

async fn handle_command(cmd: &str, db: &Db) -> String {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    if parts.is_empty() {
        return "ERROR: Empty command\r\n".to_string();
    }

    match parts[0].to_uppercase().as_str() {
        "SET" => {
            if parts.len() != 3 {
                return "ERROR: Wrong number of arguments for SET\r\n".to_string();
            }
            let mut store = db.lock().await;
            store.insert(parts[1].to_string(), parts[2].to_string());
            "OK\r\n".to_string()
        }
        "GET" => {
            if parts.len() != 2 {
                return "ERROR: Wrong number of arguments for GET\r\n".to_string();
            }
            let store = db.lock().await;
            match store.get(parts[1]) {
                Some(value) => format!("${}\r\n{}\r\n", value.len(), value),
                None => "$-1\r\n".to_string(),
            }
        }
        _ => "ERROR: Unknown command\r\n".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_command() {
        let db: Db = Arc::new(Mutex::new(HashMap::new()));
        
        // Test SET command
        let response = handle_command("SET key1 value1", &db).await;
        assert_eq!(response, "OK\r\n");
        
        // Test GET command
        let response = handle_command("GET key1", &db).await;
        assert_eq!(response, "$6\r\nvalue1\r\n");
        
        // Test GET for non-existent key
        let response = handle_command("GET nonexistent", &db).await;
        assert_eq!(response, "$-1\r\n");
    }
}
