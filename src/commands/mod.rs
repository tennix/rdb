use std::str::FromStr;
use thiserror::Error;
use crate::storage::Db;

#[derive(Debug, PartialEq)]
pub enum Command {
    Set(String, String),
    Get(String),
    Info,
    CmdInfo,
    Memory,
    Save,
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("invalid command format")]
    InvalidFormat,
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("wrong number of arguments for command")]
    WrongNumberOfArguments,
}

impl FromStr for Command {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.split("\r\n").collect();
        if lines.is_empty() {
            return Err(CommandError::InvalidFormat);
        }

        if !lines[0].starts_with('*') {
            return Err(CommandError::InvalidFormat);
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
            return Err(CommandError::InvalidFormat);
        }

        match args[0].to_uppercase().as_str() {
            "SET" => {
                if args.len() != 3 {
                    return Err(CommandError::WrongNumberOfArguments);
                }
                Ok(Command::Set(args[1].to_string(), args[2].to_string()))
            }
            "GET" => {
                if args.len() != 2 {
                    return Err(CommandError::WrongNumberOfArguments);
                }
                Ok(Command::Get(args[1].to_string()))
            }
            "INFO" => Ok(Command::Info),
            "COMMAND" => Ok(Command::CmdInfo),
            "MEMORY" => Ok(Command::Memory),
            "SAVE" => Ok(Command::Save),
            cmd => Err(CommandError::UnknownCommand(cmd.to_string())),
        }
    }
}

use crate::protocol::RespValue;

pub async fn handle_command(cmd: &str, db: &Db) -> RespValue {
    let command = match Command::from_str(cmd) {
        Ok(cmd) => cmd,
        Err(e) => return RespValue::Error(e.to_string()),
    };

    match command {
        Command::Set(key, value) => {
            let mut store = db.lock().await;
            if store.insert(key, value) {
                RespValue::SimpleString("OK".to_string())
            } else {
                RespValue::Error("ERR max memory limit exceeded".to_string())
            }
        }
        Command::Get(key) => {
            let store = db.lock().await;
            match store.get(&key) {
                Some(value) => RespValue::BulkString(Some(value.clone())),
                None => RespValue::BulkString(None),
            }
        }
        Command::CmdInfo => RespValue::Array(vec![]),
        Command::Info => {
            let store = db.lock().await;
            let info = format!(
                "# Server\r\nredis_version:1.0.0\r\n\
                # Memory\r\nused_memory:{}\r\n\
                persistence_enabled:{}\r\n",
                store.memory_usage(),
                store.is_persistence_enabled()
            );
            RespValue::BulkString(Some(info))
        }
        Command::Memory => {
            let store = db.lock().await;
            RespValue::Integer(store.memory_usage() as i64)
        }
        Command::Save => {
            let store = db.lock().await;
            match store.save_to_disk() {
                Ok(_) => RespValue::SimpleString("OK".to_string()),
                Err(e) => RespValue::Error(format!("ERR saving to disk: {}", e)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn test_command_parsing() {
        assert_eq!(
            Command::from_str("*3\r\n$3\r\nSET\r\n$4\r\nkey1\r\n$6\r\nvalue1\r\n").unwrap(),
            Command::Set("key1".to_string(), "value1".to_string())
        );

        assert_eq!(
            Command::from_str("*2\r\n$3\r\nGET\r\n$4\r\nkey1\r\n").unwrap(),
            Command::Get("key1".to_string())
        );
    }

    #[tokio::test]
    async fn test_handle_command() {
        let db: Db = Arc::new(Mutex::new(HashMap::new()));
        
        let response = handle_command("*3\r\n$3\r\nSET\r\n$4\r\nkey1\r\n$6\r\nvalue1\r\n", &db).await;
        assert_eq!(response, RespValue::SimpleString("OK".to_string()));
        
        let response = handle_command("*2\r\n$3\r\nGET\r\n$4\r\nkey1\r\n", &db).await;
        assert_eq!(response, RespValue::BulkString(Some("value1".to_string())));
        
        let response = handle_command("*2\r\n$3\r\nGET\r\n$10\r\nnonexistent\r\n", &db).await;
        assert_eq!(response, RespValue::BulkString(None));
    }
}
