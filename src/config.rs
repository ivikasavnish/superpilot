use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub listen_addr: SocketAddr,
    pub target_addr: SocketAddr,
    pub mode: ProxyMode,
    pub metrics_addr: SocketAddr,
    pub tcp: TcpConfig,
    pub udp: UdpConfig,
    pub http: HttpConfig,
    #[cfg(feature = "recorder")]
    pub recorder: RecorderConfig,
    pub safety: SafetyConfig,
    #[serde(default)]
    pub prometheus: PrometheusConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    Tcp,
    Udp,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpConfig {
    pub buffer_size: usize,
    pub connection_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpConfig {
    pub buffer_size: usize,
    pub max_packet_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpConfig {
    pub max_header_size: usize,
    pub max_body_size: usize,
}

#[cfg(feature = "recorder")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecorderConfig {
    pub enabled: bool,
    pub mode: RecordingMode,
    pub buffer_size: usize,
    pub output_path: String,
    pub tls_mitm: bool,
    pub ca_cert_path: Option<String>,
    pub ca_key_path: Option<String>,
}

#[cfg(feature = "recorder")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingMode {
    Off,
    Metadata,
    Payload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub max_memory_mb: usize,
    pub max_latency_ms: u64,
    pub rate_limit_requests_per_sec: u64,
    pub sampling_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    pub push_gateway_url: Option<String>,
    pub push_interval_secs: u64,
    pub job_name: String,
    pub instance: String,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        PrometheusConfig {
            push_gateway_url: None,
            push_interval_secs: 60,
            job_name: "superpilot".to_string(),
            instance: "localhost".to_string(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            listen_addr: "0.0.0.0:8080".parse().unwrap(),
            target_addr: "127.0.0.1:8081".parse().unwrap(),
            mode: ProxyMode::Tcp,
            metrics_addr: "0.0.0.0:9090".parse().unwrap(),
            tcp: TcpConfig {
                buffer_size: 8192,
                connection_timeout_secs: 300,
            },
            udp: UdpConfig {
                buffer_size: 65536,
                max_packet_size: 65507,
            },
            http: HttpConfig {
                max_header_size: 8192,
                max_body_size: 1024 * 1024,
            },
            #[cfg(feature = "recorder")]
            recorder: RecorderConfig {
                enabled: false,
                mode: RecordingMode::Off,
                buffer_size: 1024 * 1024,
                output_path: "/tmp/recordings".to_string(),
                tls_mitm: false,
                ca_cert_path: None,
                ca_key_path: None,
            },
            safety: SafetyConfig {
                max_memory_mb: 256,
                max_latency_ms: 100,
                rate_limit_requests_per_sec: 10000,
                sampling_rate: 1.0,
            },
            prometheus: PrometheusConfig::default(),
        }
    }
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    pub fn from_env() -> Self {
        let mut config = Config::default();
        
        if let Ok(addr) = std::env::var("LISTEN_ADDR") {
            if let Ok(parsed) = addr.parse() {
                config.listen_addr = parsed;
            }
        }
        
        if let Ok(addr) = std::env::var("TARGET_ADDR") {
            if let Ok(parsed) = addr.parse() {
                config.target_addr = parsed;
            }
        }
        
        if let Ok(mode) = std::env::var("PROXY_MODE") {
            config.mode = match mode.to_lowercase().as_str() {
                "tcp" => ProxyMode::Tcp,
                "udp" => ProxyMode::Udp,
                "http" => ProxyMode::Http,
                _ => ProxyMode::Tcp,
            };
        }
        
        config
    }
}
