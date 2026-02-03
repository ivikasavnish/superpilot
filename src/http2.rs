use std::sync::Arc;
use crate::config::Config;
use crate::metrics::Metrics;

/// HTTP/2 observer (observe-only)
/// Detects HTTP/2 via ALPN negotiation
/// Tracks streams, resets, and bytes without parsing frames
pub struct Http2Observer {
    config: Arc<Config>,
    metrics: Arc<Metrics>,
}

impl Http2Observer {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics>) -> Self {
        Http2Observer { config, metrics }
    }

    /// Detect HTTP/2 from connection preface
    /// HTTP/2 starts with: "PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"
    pub fn is_http2(data: &[u8]) -> bool {
        const PREFACE: &[u8] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
        data.len() >= PREFACE.len() && &data[..PREFACE.len()] == PREFACE
    }

    /// Observe HTTP/2 frames (minimal parsing)
    /// Only tracks frame types and counts for metrics
    pub fn observe_frame(&self, data: &[u8]) {
        if data.len() < 9 {
            return; // Invalid frame
        }

        // Frame format: 3-byte length, 1-byte type, 1-byte flags, 4-byte stream ID
        let frame_type = data[3];

        match frame_type {
            0x0 => {
                // DATA frame
                self.metrics.http2_bytes_total.inc_by(data.len() as u64);
            }
            0x1 => {
                // HEADERS frame (new stream)
                self.metrics.http2_streams_total.inc();
            }
            0x3 => {
                // RST_STREAM frame
                self.metrics.http2_stream_resets.inc();
            }
            _ => {
                // Other frame types
                self.metrics.http2_bytes_total.inc_by(data.len() as u64);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http2_detection() {
        let preface = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";
        assert!(Http2Observer::is_http2(preface));
        assert!(!Http2Observer::is_http2(b"GET / HTTP/1.1\r\n"));
    }

    #[test]
    fn test_http2_observer_creation() {
        let config = Arc::new(Config::default());
        let metrics = Metrics::new().unwrap();
        let observer = Http2Observer::new(config, metrics);
        assert!(true); // Basic creation test
    }
}
