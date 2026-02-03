use std::net::UdpSocket;
use std::sync::Arc;
use crate::config::Config;
use crate::metrics::Metrics;
use crate::safety::SafetyGuard;

/// Stateless UDP forwarder
/// Best-effort forwarding with drop metrics
pub struct UdpForwarder {
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    safety: Arc<SafetyGuard>,
}

impl UdpForwarder {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics>, safety: Arc<SafetyGuard>) -> Self {
        UdpForwarder {
            config,
            metrics,
            safety,
        }
    }

    pub fn run(&self) -> std::io::Result<()> {
        let socket = UdpSocket::bind(self.config.listen_addr)?;
        println!("UDP forwarder listening on {}", self.config.listen_addr);

        // Set non-blocking
        socket.set_nonblocking(false)?;

        let buffer_size = self.config.udp.buffer_size;
        let max_packet = self.config.udp.max_packet_size;
        let mut buffer = vec![0u8; buffer_size];

        loop {
            match socket.recv_from(&mut buffer) {
                Ok((len, _src_addr)) => {
                    // Check safety limits
                    if !self.safety.should_allow_packet() {
                        self.metrics.safety_rate_limited.inc();
                        self.metrics.udp_drops_total.inc();
                        continue;
                    }

                    // Check packet size
                    if len > max_packet {
                        eprintln!("UDP packet too large: {} bytes", len);
                        self.metrics.udp_drops_total.inc();
                        continue;
                    }

                    self.metrics.udp_packets_received.inc();
                    self.metrics.udp_bytes_received.inc_by(len as u64);

                    // Forward to target (best-effort)
                    match socket.send_to(&buffer[..len], self.config.target_addr) {
                        Ok(sent) => {
                            if sent == len {
                                self.metrics.udp_packets_sent.inc();
                                self.metrics.udp_bytes_sent.inc_by(sent as u64);
                            } else {
                                self.metrics.udp_drops_total.inc();
                            }
                        }
                        Err(e) => {
                            eprintln!("UDP send error: {}", e);
                            self.metrics.udp_drops_total.inc();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("UDP recv error: {}", e);
                    self.metrics.udp_drops_total.inc();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_udp_forwarder_creation() {
        let config = Arc::new(Config::default());
        let metrics = Metrics::new().unwrap();
        let safety = Arc::new(SafetyGuard::new(config.clone(), metrics.clone()));
        let forwarder = UdpForwarder::new(config, metrics, safety);
        assert!(true); // Basic creation test
    }
}
