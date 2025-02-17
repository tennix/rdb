//! RESP (Redis Serialization Protocol) implementation
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<String>),
    Array(Vec<RespValue>),
}

#[derive(Error, Debug)]
pub enum RespError {
    #[error("invalid RESP format")]
    InvalidFormat,
    #[error("incomplete input")]
    Incomplete,
    #[error("invalid length: {0}")]
    InvalidLength(String),
}

impl RespValue {
    pub fn to_string(&self) -> String {
        match self {
            RespValue::SimpleString(s) => format!("+{}\r\n", s),
            RespValue::Error(msg) => format!("-{}\r\n", msg),
            RespValue::Integer(n) => format!(":{}\r\n", n),
            RespValue::BulkString(None) => "$-1\r\n".to_string(),
            RespValue::BulkString(Some(s)) => format!("${}\r\n{}\r\n", s.len(), s),
            RespValue::Array(items) => {
                let mut result = format!("*{}\r\n", items.len());
                for item in items {
                    result.push_str(&item.to_string());
                }
                result
            }
        }
    }
}

pub fn parse_resp(input: &str) -> Result<(RespValue, usize), RespError> {
    if input.is_empty() {
        return Err(RespError::Incomplete);
    }

    match input.chars().next().unwrap() {
        '+' => parse_simple_string(input),
        '-' => parse_error(input),
        ':' => parse_integer(input),
        '$' => parse_bulk_string(input),
        '*' => parse_array(input),
        _ => Err(RespError::InvalidFormat),
    }
}

fn parse_simple_string(input: &str) -> Result<(RespValue, usize), RespError> {
    if let Some(end) = input[1..].find("\r\n") {
        Ok((
            RespValue::SimpleString(input[1..=end].to_string()),
            end + 3,
        ))
    } else {
        Err(RespError::Incomplete)
    }
}

fn parse_error(input: &str) -> Result<(RespValue, usize), RespError> {
    if let Some(end) = input[1..].find("\r\n") {
        Ok((
            RespValue::Error(input[1..=end].to_string()),
            end + 3,
        ))
    } else {
        Err(RespError::Incomplete)
    }
}

fn parse_integer(input: &str) -> Result<(RespValue, usize), RespError> {
    if let Some(end) = input[1..].find("\r\n") {
        let num = input[1..=end].parse::<i64>()
            .map_err(|_| RespError::InvalidFormat)?;
        Ok((RespValue::Integer(num), end + 3))
    } else {
        Err(RespError::Incomplete)
    }
}

fn parse_bulk_string(input: &str) -> Result<(RespValue, usize), RespError> {
    if let Some(len_end) = input[1..].find("\r\n") {
        let length = input[1..=len_end].parse::<i64>()
            .map_err(|_| RespError::InvalidFormat)?;
        
        if length == -1 {
            return Ok((RespValue::BulkString(None), len_end + 3));
        }
        
        let start = len_end + 3;
        let end = start + length as usize;
        
        if input.len() < end + 2 {
            return Err(RespError::Incomplete);
        }
        
        if &input[end..end + 2] != "\r\n" {
            return Err(RespError::InvalidFormat);
        }
        
        Ok((
            RespValue::BulkString(Some(input[start..end].to_string())),
            end + 2,
        ))
    } else {
        Err(RespError::Incomplete)
    }
}

fn parse_array(input: &str) -> Result<(RespValue, usize), RespError> {
    if let Some(len_end) = input[1..].find("\r\n") {
        let length = input[1..=len_end].parse::<i64>()
            .map_err(|_| RespError::InvalidFormat)?;
        
        if length == -1 {
            return Ok((RespValue::Array(vec![]), len_end + 3));
        }
        
        let mut pos = len_end + 3;
        let mut items = Vec::new();
        
        for _ in 0..length {
            if pos >= input.len() {
                return Err(RespError::Incomplete);
            }
            
            let (value, len) = parse_resp(&input[pos..])?;
            items.push(value);
            pos += len;
        }
        
        Ok((RespValue::Array(items), pos))
    } else {
        Err(RespError::Incomplete)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_string() {
        let input = "+OK\r\n";
        let (value, _) = parse_resp(input).unwrap();
        assert_eq!(value, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_parse_error() {
        let input = "-Error message\r\n";
        let (value, _) = parse_resp(input).unwrap();
        assert_eq!(value, RespValue::Error("Error message".to_string()));
    }

    #[test]
    fn test_parse_integer() {
        let input = ":1000\r\n";
        let (value, _) = parse_resp(input).unwrap();
        assert_eq!(value, RespValue::Integer(1000));
    }

    #[test]
    fn test_parse_bulk_string() {
        let input = "$5\r\nhello\r\n";
        let (value, _) = parse_resp(input).unwrap();
        assert_eq!(value, RespValue::BulkString(Some("hello".to_string())));
    }

    #[test]
    fn test_parse_null_bulk_string() {
        let input = "$-1\r\n";
        let (value, _) = parse_resp(input).unwrap();
        assert_eq!(value, RespValue::BulkString(None));
    }

    #[test]
    fn test_parse_array() {
        let input = "*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let (value, _) = parse_resp(input).unwrap();
        match value {
            RespValue::Array(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0], RespValue::BulkString(Some("hello".to_string())));
                assert_eq!(items[1], RespValue::BulkString(Some("world".to_string())));
            }
            _ => panic!("Expected array"),
        }
    }
}
