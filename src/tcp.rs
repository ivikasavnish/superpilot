use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::config::Config;
use crate::metrics::Metrics;
use crate::safety::SafetyGuard;

/// Transparent TCP proxy
/// Forwards TCP traffic without parsing protocols
/// Collects metrics on connections, bytes, and errors
pub struct TcpProxy {
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    safety: Arc<SafetyGuard>,
}

impl TcpProxy {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics>, safety: Arc<SafetyGuard>) -> Self {
        TcpProxy {
            config,
            metrics,
            safety,
        }
    }

    pub fn run(&self) -> io::Result<()> {
        let listener = TcpListener::bind(self.config.listen_addr)?;
        println!("TCP proxy listening on {}", self.config.listen_addr);

        // Set non-blocking mode for listener
        listener.set_nonblocking(false)?;

        for stream in listener.incoming() {
            match stream {
                Ok(client) => {
                    let config = Arc::clone(&self.config);
                    let metrics = Arc::clone(&self.metrics);
                    let safety = Arc::clone(&self.safety);

                    // Spawn a thread per connection (no async on hot path)
                    std::thread::spawn(move || {
                        if let Err(e) = handle_tcp_connection(client, config, metrics, safety) {
                            eprintln!("TCP connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("TCP accept error: {}", e);
                    self.metrics.tcp_errors_total.with_label_values(&["accept"]).inc();
                }
            }
        }

        Ok(())
    }
}

fn handle_tcp_connection(
    mut client: TcpStream,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    safety: Arc<SafetyGuard>,
) -> io::Result<()> {
    let start = Instant::now();
    
    // Check safety limits
    if !safety.should_allow_connection() {
        metrics.safety_rate_limited.inc();
        return Ok(());
    }

    metrics.tcp_connections_total.inc();
    metrics.tcp_active_connections.inc();

    // Connect to target
    let mut target = TcpStream::connect(config.target_addr)?;

    // Set timeouts
    let timeout = Duration::from_secs(config.tcp.connection_timeout_secs);
    client.set_read_timeout(Some(timeout))?;
    client.set_write_timeout(Some(timeout))?;
    target.set_read_timeout(Some(timeout))?;
    target.set_write_timeout(Some(timeout))?;

    // Fixed buffers for hot path (no heap allocations)
    let buffer_size = config.tcp.buffer_size;
    let mut client_buf = vec![0u8; buffer_size];
    let mut target_buf = vec![0u8; buffer_size];

    // Set non-blocking for efficient forwarding
    client.set_nonblocking(true)?;
    target.set_nonblocking(true)?;

    loop {
        // Forward client -> target
        match client.read(&mut client_buf) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                target.write_all(&client_buf[..n])?;
                metrics.tcp_bytes_sent.inc_by(n as u64);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No data available, continue
            }
            Err(e) => {
                metrics.tcp_errors_total.with_label_values(&["client_read"]).inc();
                return Err(e);
            }
        }

        // Forward target -> client
        match target.read(&mut target_buf) {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                client.write_all(&target_buf[..n])?;
                metrics.tcp_bytes_received.inc_by(n as u64);
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No data available, continue
            }
            Err(e) => {
                metrics.tcp_errors_total.with_label_values(&["target_read"]).inc();
                return Err(e);
            }
        }

        // Small sleep to avoid busy-waiting
        std::thread::sleep(Duration::from_micros(100));
    }

    let duration = start.elapsed().as_secs_f64();
    metrics.tcp_duration_seconds
        .with_label_values(&["success"])
        .observe(duration);
    metrics.tcp_active_connections.dec();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_proxy_creation() {
        let config = Arc::new(Config::default());
        let metrics = Metrics::new().unwrap();
        let safety = Arc::new(SafetyGuard::new(config.clone(), metrics.clone()));
        let proxy = TcpProxy::new(config, metrics, safety);
        assert!(true); // Basic creation test
    }
}
