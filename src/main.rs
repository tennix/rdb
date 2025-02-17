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

async fn handle_command(cmd: &str, db: &Db) -> String {
    let lines: Vec<&str> = cmd.split("\r\n").collect();
    if lines.is_empty() {
        return "-ERR empty command\r\n".to_string();
    }

    // Parse RESP array format
    if !lines[0].starts_with('*') {
        return "-ERR invalid RESP format\r\n".to_string();
    }

    let mut args = Vec::new();
    let mut i = 1;
    while i < lines.len() {
        if lines[i].starts_with('$') {
            if i + 1 < lines.len() {
                args.push(lines[i + 1]);
                i += 2;
            }
        } else {
            i += 1;
        }
    }

    if args.is_empty() {
        return "-ERR empty command\r\n".to_string();
    }

    match args[0].to_uppercase().as_str() {
        "SET" => {
            if args.len() != 3 {
                return "-ERR wrong number of arguments for 'set' command\r\n".to_string();
            }
            let mut store = db.lock().await;
            store.insert(args[1].to_string(), args[2].to_string());
            "+OK\r\n".to_string()
        }
        "GET" => {
            if args.len() != 2 {
                return "-ERR wrong number of arguments for 'get' command\r\n".to_string();
            }
            let store = db.lock().await;
            match store.get(args[1]) {
                Some(value) => format!("${}\r\n{}\r\n", value.len(), value),
                None => "$-1\r\n".to_string(),
            }
        }
        "COMMAND" => {
            if args.len() == 1 {
                // Return empty array for COMMAND
                "*0\r\n".to_string()
            } else {
                "*-1\r\n".to_string()
            }
        }
        "INFO" => {
            // Return minimal server info
            let info = "# Server\r\nredis_version:1.0.0\r\n";
            format!("${}\r\n{}\r\n", info.len(), info)
        }
        _ => "-ERR unknown command\r\n".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_command() {
        let db: Db = Arc::new(Mutex::new(HashMap::new()));
        
        // Test SET command
        let response = handle_command("*3\r\n$3\r\nSET\r\n$4\r\nkey1\r\n$6\r\nvalue1\r\n", &db).await;
        assert_eq!(response, "+OK\r\n");
        
        // Test GET command
        let response = handle_command("*2\r\n$3\r\nGET\r\n$4\r\nkey1\r\n", &db).await;
        assert_eq!(response, "$6\r\nvalue1\r\n");
        
        // Test GET for non-existent key
        let response = handle_command("*2\r\n$3\r\nGET\r\n$10\r\nnonexistent\r\n", &db).await;
        assert_eq!(response, "$-1\r\n");
    }
}
