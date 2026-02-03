mod config;
mod metrics;
mod tcp;
mod udp;
mod http1;
mod http2;
mod quic;
mod mitm;
mod recorder;
mod safety;
mod util;

use std::sync::Arc;
use std::thread;
use config::{Config, ProxyMode};
use metrics::Metrics;
use safety::SafetyGuard;

fn main() {
    // Print build information
    #[cfg(feature = "prod")]
    println!("Superpilot - Production Mode (Observe-Only)");
    
    #[cfg(feature = "recorder")]
    println!("Superpilot - Recorder Mode (MITM + Capture)");
    
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!("Build: 2021 edition");

    // Load configuration
    let config = if let Ok(config_path) = std::env::var("CONFIG_FILE") {
        Config::from_file(&config_path)
            .unwrap_or_else(|e| {
                eprintln!("Failed to load config from {}: {}", config_path, e);
                eprintln!("Using default configuration");
                Config::default()
            })
    } else {
        Config::from_env()
    };

    let config = Arc::new(config);
    println!("Configuration loaded");
    println!("  Listen: {}", config.listen_addr);
    println!("  Target: {}", config.target_addr);
    println!("  Mode: {:?}", config.mode);

    // Initialize metrics
    let metrics = Metrics::new().expect("Failed to initialize metrics");
    println!("Metrics initialized on {}", config.metrics_addr);

    // Initialize safety guard
    let safety = Arc::new(SafetyGuard::new(Arc::clone(&config), Arc::clone(&metrics)));
    println!("Safety guard initialized");

    // Start metrics server in background thread
    let metrics_clone = Arc::clone(&metrics);
    let metrics_addr = config.metrics_addr;
    thread::spawn(move || {
        start_metrics_server(metrics_clone, metrics_addr);
    });

    // Start proxy based on mode
    match config.mode {
        ProxyMode::Tcp => {
            println!("Starting TCP proxy...");
            let proxy = tcp::TcpProxy::new(
                Arc::clone(&config),
                Arc::clone(&metrics),
                Arc::clone(&safety),
            );
            if let Err(e) = proxy.run() {
                eprintln!("TCP proxy error: {}", e);
                std::process::exit(1);
            }
        }
        ProxyMode::Udp => {
            println!("Starting UDP forwarder...");
            let forwarder = udp::UdpForwarder::new(
                Arc::clone(&config),
                Arc::clone(&metrics),
                Arc::clone(&safety),
            );
            if let Err(e) = forwarder.run() {
                eprintln!("UDP forwarder error: {}", e);
                std::process::exit(1);
            }
        }
        ProxyMode::Http => {
            println!("Starting HTTP proxy...");
            // HTTP proxy would use HTTP/1 observer
            // For now, fallback to TCP
            let proxy = tcp::TcpProxy::new(
                Arc::clone(&config),
                Arc::clone(&metrics),
                Arc::clone(&safety),
            );
            if let Err(e) = proxy.run() {
                eprintln!("HTTP proxy error: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn start_metrics_server(metrics: Arc<Metrics>, addr: std::net::SocketAddr) {
    use std::io::Write;
    use std::net::TcpListener;
    use prometheus::Encoder;

    let listener = TcpListener::bind(addr).expect("Failed to bind metrics server");
    println!("Metrics server listening on {}", addr);

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            // Encode metrics
            let encoder = prometheus::TextEncoder::new();
            let metric_families = metrics.registry.gather();
            let mut buffer = Vec::new();
            
            if encoder.encode(&metric_families, &mut buffer).is_ok() {
                // Send HTTP response
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n",
                    buffer.len()
                );
                
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&buffer);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.listen_addr.port(), 8080);
        assert_eq!(config.target_addr.port(), 8081);
    }

    #[test]
    fn test_metrics_initialization() {
        let metrics = Metrics::new();
        assert!(metrics.is_ok());
    }
}
