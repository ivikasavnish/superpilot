use prometheus::{
    Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, Opts, Registry,
};
use std::sync::Arc;

/// Metrics collector for observability
pub struct Metrics {
    pub registry: Registry,
    
    // TCP metrics
    pub tcp_connections_total: IntCounter,
    pub tcp_bytes_sent: IntCounter,
    pub tcp_bytes_received: IntCounter,
    pub tcp_active_connections: IntGauge,
    pub tcp_errors_total: IntCounterVec,
    pub tcp_duration_seconds: HistogramVec,
    
    // UDP metrics
    pub udp_packets_sent: IntCounter,
    pub udp_packets_received: IntCounter,
    pub udp_bytes_sent: IntCounter,
    pub udp_bytes_received: IntCounter,
    pub udp_drops_total: IntCounter,
    
    // HTTP metrics
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_request_size_bytes: Histogram,
    pub http_response_size_bytes: Histogram,
    
    // HTTP/2 metrics
    pub http2_streams_total: IntCounter,
    pub http2_stream_resets: IntCounter,
    pub http2_bytes_total: IntCounter,
    
    // QUIC/HTTP/3 metrics
    pub quic_packets_total: IntCounter,
    pub quic_bytes_total: IntCounter,
    pub quic_rtt_milliseconds: Histogram,
    
    // Recording metrics (only when feature enabled)
    #[cfg(feature = "recorder")]
    pub recorder_events_total: IntCounter,
    #[cfg(feature = "recorder")]
    pub recorder_bytes_written: IntCounter,
    #[cfg(feature = "recorder")]
    pub recorder_drops_total: IntCounter,
    #[cfg(feature = "recorder")]
    pub recorder_auto_disabled: IntCounter,
    
    // Safety metrics
    pub safety_rate_limited: IntCounter,
    pub safety_sampled_out: IntCounter,
    pub memory_usage_bytes: IntGauge,
}

impl Metrics {
    pub fn new() -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let registry = Registry::new();
        
        // TCP metrics
        let tcp_connections_total = IntCounter::new(
            "tcp_connections_total",
            "Total number of TCP connections",
        )?;
        let tcp_bytes_sent = IntCounter::new(
            "tcp_bytes_sent_total",
            "Total bytes sent over TCP",
        )?;
        let tcp_bytes_received = IntCounter::new(
            "tcp_bytes_received_total",
            "Total bytes received over TCP",
        )?;
        let tcp_active_connections = IntGauge::new(
            "tcp_active_connections",
            "Number of active TCP connections",
        )?;
        let tcp_errors_total = IntCounterVec::new(
            Opts::new("tcp_errors_total", "Total TCP errors by type"),
            &["error_type"],
        )?;
        let tcp_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "tcp_connection_duration_seconds",
                "TCP connection duration",
            ),
            &["status"],
        )?;
        
        // UDP metrics
        let udp_packets_sent = IntCounter::new(
            "udp_packets_sent_total",
            "Total UDP packets sent",
        )?;
        let udp_packets_received = IntCounter::new(
            "udp_packets_received_total",
            "Total UDP packets received",
        )?;
        let udp_bytes_sent = IntCounter::new(
            "udp_bytes_sent_total",
            "Total bytes sent over UDP",
        )?;
        let udp_bytes_received = IntCounter::new(
            "udp_bytes_received_total",
            "Total bytes received over UDP",
        )?;
        let udp_drops_total = IntCounter::new(
            "udp_drops_total",
            "Total UDP packet drops",
        )?;
        
        // HTTP metrics
        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests"),
            &["method", "status"],
        )?;
        let http_request_duration_seconds = HistogramVec::new(
            prometheus::HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration",
            ),
            &["method", "status"],
        )?;
        let http_request_size_bytes = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "http_request_size_bytes",
                "HTTP request size",
            ),
        )?;
        let http_response_size_bytes = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "http_response_size_bytes",
                "HTTP response size",
            ),
        )?;
        
        // HTTP/2 metrics
        let http2_streams_total = IntCounter::new(
            "http2_streams_total",
            "Total HTTP/2 streams",
        )?;
        let http2_stream_resets = IntCounter::new(
            "http2_stream_resets_total",
            "Total HTTP/2 stream resets",
        )?;
        let http2_bytes_total = IntCounter::new(
            "http2_bytes_total",
            "Total HTTP/2 bytes",
        )?;
        
        // QUIC/HTTP/3 metrics
        let quic_packets_total = IntCounter::new(
            "quic_packets_total",
            "Total QUIC packets",
        )?;
        let quic_bytes_total = IntCounter::new(
            "quic_bytes_total",
            "Total QUIC bytes",
        )?;
        let quic_rtt_milliseconds = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "quic_rtt_milliseconds",
                "QUIC RTT",
            ),
        )?;
        
        // Recording metrics
        #[cfg(feature = "recorder")]
        let recorder_events_total = IntCounter::new(
            "recorder_events_total",
            "Total recorded events",
        )?;
        #[cfg(feature = "recorder")]
        let recorder_bytes_written = IntCounter::new(
            "recorder_bytes_written_total",
            "Total bytes written by recorder",
        )?;
        #[cfg(feature = "recorder")]
        let recorder_drops_total = IntCounter::new(
            "recorder_drops_total",
            "Total dropped recorder events",
        )?;
        #[cfg(feature = "recorder")]
        let recorder_auto_disabled = IntCounter::new(
            "recorder_auto_disabled_total",
            "Times recorder was auto-disabled",
        )?;
        
        // Safety metrics
        let safety_rate_limited = IntCounter::new(
            "safety_rate_limited_total",
            "Total rate-limited requests",
        )?;
        let safety_sampled_out = IntCounter::new(
            "safety_sampled_out_total",
            "Total sampled-out requests",
        )?;
        let memory_usage_bytes = IntGauge::new(
            "memory_usage_bytes",
            "Current memory usage",
        )?;
        
        // Register all metrics
        registry.register(Box::new(tcp_connections_total.clone()))?;
        registry.register(Box::new(tcp_bytes_sent.clone()))?;
        registry.register(Box::new(tcp_bytes_received.clone()))?;
        registry.register(Box::new(tcp_active_connections.clone()))?;
        registry.register(Box::new(tcp_errors_total.clone()))?;
        registry.register(Box::new(tcp_duration_seconds.clone()))?;
        
        registry.register(Box::new(udp_packets_sent.clone()))?;
        registry.register(Box::new(udp_packets_received.clone()))?;
        registry.register(Box::new(udp_bytes_sent.clone()))?;
        registry.register(Box::new(udp_bytes_received.clone()))?;
        registry.register(Box::new(udp_drops_total.clone()))?;
        
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration_seconds.clone()))?;
        registry.register(Box::new(http_request_size_bytes.clone()))?;
        registry.register(Box::new(http_response_size_bytes.clone()))?;
        
        registry.register(Box::new(http2_streams_total.clone()))?;
        registry.register(Box::new(http2_stream_resets.clone()))?;
        registry.register(Box::new(http2_bytes_total.clone()))?;
        
        registry.register(Box::new(quic_packets_total.clone()))?;
        registry.register(Box::new(quic_bytes_total.clone()))?;
        registry.register(Box::new(quic_rtt_milliseconds.clone()))?;
        
        #[cfg(feature = "recorder")]
        {
            registry.register(Box::new(recorder_events_total.clone()))?;
            registry.register(Box::new(recorder_bytes_written.clone()))?;
            registry.register(Box::new(recorder_drops_total.clone()))?;
            registry.register(Box::new(recorder_auto_disabled.clone()))?;
        }
        
        registry.register(Box::new(safety_rate_limited.clone()))?;
        registry.register(Box::new(safety_sampled_out.clone()))?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;
        
        Ok(Arc::new(Metrics {
            registry,
            tcp_connections_total,
            tcp_bytes_sent,
            tcp_bytes_received,
            tcp_active_connections,
            tcp_errors_total,
            tcp_duration_seconds,
            udp_packets_sent,
            udp_packets_received,
            udp_bytes_sent,
            udp_bytes_received,
            udp_drops_total,
            http_requests_total,
            http_request_duration_seconds,
            http_request_size_bytes,
            http_response_size_bytes,
            http2_streams_total,
            http2_stream_resets,
            http2_bytes_total,
            quic_packets_total,
            quic_bytes_total,
            quic_rtt_milliseconds,
            #[cfg(feature = "recorder")]
            recorder_events_total,
            #[cfg(feature = "recorder")]
            recorder_bytes_written,
            #[cfg(feature = "recorder")]
            recorder_drops_total,
            #[cfg(feature = "recorder")]
            recorder_auto_disabled,
            safety_rate_limited,
            safety_sampled_out,
            memory_usage_bytes,
        }))
    }
}
