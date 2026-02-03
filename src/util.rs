/// Utility functions for the proxy
use std::time::{SystemTime, UNIX_EPOCH};

/// Get current timestamp in seconds
pub fn current_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Get current timestamp in milliseconds
pub fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

/// Parse socket address from string with default port
pub fn parse_addr_with_default(addr: &str, default_port: u16) -> Result<std::net::SocketAddr, String> {
    // Check if port is included
    if addr.contains(':') {
        addr.parse().map_err(|e| format!("Invalid address: {}", e))
    } else {
        // Add default port
        let with_port = format!("{}:{}", addr, default_port);
        with_port.parse().map_err(|e| format!("Invalid address: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_parse_addr_with_default() {
        let addr = parse_addr_with_default("127.0.0.1:8080", 9090).unwrap();
        assert_eq!(addr.port(), 8080);

        let addr = parse_addr_with_default("127.0.0.1", 9090).unwrap();
        assert_eq!(addr.port(), 9090);
    }

    #[test]
    fn test_timestamps() {
        let secs = current_timestamp_secs();
        let millis = current_timestamp_millis();
        
        assert!(secs > 0);
        assert!(millis > 0);
        assert!(millis > secs * 1000);
    }
}
