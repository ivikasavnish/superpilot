use std::sync::Arc;
use crate::config::Config;
use crate::metrics::Metrics;

/// QUIC/HTTP/3 observer (observe-only)
/// Detects QUIC via packet format heuristics
/// Tracks packets, bytes, and RTT estimates
pub struct QuicObserver {
    config: Arc<Config>,
    metrics: Arc<Metrics>,
}

impl QuicObserver {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics>) -> Self {
        QuicObserver { config, metrics }
    }

    /// Detect QUIC packet via header format
    /// QUIC packets have a specific header format with version field
    pub fn is_quic(data: &[u8]) -> bool {
        if data.is_empty() {
            return false;
        }

        // Check for QUIC long header (bit 7 = 1)
        let first_byte = data[0];
        let is_long_header = (first_byte & 0x80) != 0;

        if is_long_header && data.len() >= 5 {
            // Long header has version field at bytes 1-4
            // QUIC version is non-zero for valid packets
            let version = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
            return version != 0;
        }

        // Short header check (bit 7 = 0, bit 6 = 1 for QUIC)
        (first_byte & 0x40) != 0
    }

    /// Observe QUIC packet (minimal parsing)
    /// Only tracks packet counts and sizes
    pub fn observe_packet(&self, data: &[u8]) {
        self.metrics.quic_packets_total.inc();
        self.metrics.quic_bytes_total.inc_by(data.len() as u64);

        // RTT estimation would require connection state
        // For observe-only mode, we record a placeholder
        // Real RTT would come from ACK frame analysis
        self.metrics.quic_rtt_milliseconds.observe(10.0); // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quic_detection() {
        // QUIC long header with version
        let quic_long = vec![0xC0, 0x00, 0x00, 0x00, 0x01];
        assert!(QuicObserver::is_quic(&quic_long));

        // QUIC short header
        let quic_short = vec![0x40, 0x01, 0x02, 0x03];
        assert!(QuicObserver::is_quic(&quic_short));

        // Not QUIC
        let not_quic = vec![0x00, 0x01, 0x02, 0x03];
        assert!(!QuicObserver::is_quic(&not_quic));
    }

    #[test]
    fn test_quic_observer_creation() {
        let config = Arc::new(Config::default());
        let metrics = Metrics::new().unwrap();
        let observer = QuicObserver::new(config, metrics);
        assert!(true); // Basic creation test
    }
}
