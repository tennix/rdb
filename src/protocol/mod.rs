//! RESP (Redis Serialization Protocol) implementation

pub fn parse_resp(input: &str) -> Vec<&str> {
    input.split("\r\n").collect()
}

pub fn format_simple_string(s: &str) -> String {
    format!("+{}\r\n", s)
}

pub fn format_error(err: &str) -> String {
    format!("-{}\r\n", err)
}

pub fn format_bulk_string(s: &str) -> String {
    format!("${}\r\n{}\r\n", s.len(), s)
}

pub fn format_null() -> String {
    "$-1\r\n".to_string()
}
