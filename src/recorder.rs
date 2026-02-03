/// Recording system with ring buffer and async writer
/// Disabled by default, drops on backpressure
#[cfg(feature = "recorder")]
use std::fs::OpenOptions;
#[cfg(feature = "recorder")]
use std::io::Write;
#[cfg(feature = "recorder")]
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
#[cfg(feature = "recorder")]
use std::sync::Arc;
#[cfg(feature = "recorder")]
use std::time::SystemTime;

#[cfg(feature = "recorder")]
use crate::config::{Config, RecordingMode};
#[cfg(feature = "recorder")]
use crate::metrics::Metrics;

#[cfg(feature = "recorder")]
#[derive(Debug, Clone)]
pub struct RecordedEvent {
    pub timestamp: u64,
    pub event_type: EventType,
    pub metadata: EventMetadata,
    pub payload: Option<Vec<u8>>,
}

#[cfg(feature = "recorder")]
#[derive(Debug, Clone)]
pub enum EventType {
    HttpRequest,
    HttpResponse,
    TcpConnection,
    UdpPacket,
}

#[cfg(feature = "recorder")]
#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub source: String,
    pub destination: String,
    pub protocol: String,
    pub method: Option<String>,
    pub path: Option<String>,
    pub status: Option<u16>,
}

#[cfg(feature = "recorder")]
pub struct Recorder {
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    enabled: AtomicBool,
    buffer: Vec<RecordedEvent>,
    buffer_pos: AtomicUsize,
    max_buffer_size: usize,
}

#[cfg(feature = "recorder")]
impl Recorder {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics>) -> Self {
        let max_buffer_size = config.recorder.buffer_size;
        
        Recorder {
            config,
            metrics,
            enabled: AtomicBool::new(false),
            buffer: Vec::with_capacity(max_buffer_size),
            buffer_pos: AtomicUsize::new(0),
            max_buffer_size,
        }
    }

    /// Check if recording is enabled (atomic check on hot path)
    #[inline(always)]
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Enable recording
    pub fn enable(&self) {
        self.enabled.store(true, Ordering::Relaxed);
    }

    /// Disable recording
    pub fn disable(&self) {
        self.enabled.store(false, Ordering::Relaxed);
    }

    /// Record an event (non-blocking, drops on overflow)
    pub fn record(&mut self, event: RecordedEvent) {
        // Fast path: check if recording is disabled
        if !self.is_enabled() {
            return;
        }

        // Check recording mode
        match self.config.recorder.mode {
            RecordingMode::Off => return,
            RecordingMode::Metadata => {
                // Record without payload
                self.record_internal(RecordedEvent {
                    payload: None,
                    ..event
                });
            }
            RecordingMode::Payload => {
                // Record with payload
                self.record_internal(event);
            }
        }
    }

    fn record_internal(&mut self, event: RecordedEvent) {
        let pos = self.buffer_pos.load(Ordering::Relaxed);
        
        if pos >= self.max_buffer_size {
            // Buffer full, drop event (never block)
            self.metrics.recorder_drops_total.inc();
            return;
        }

        // Add to ring buffer
        if pos < self.buffer.len() {
            self.buffer[pos] = event;
        } else {
            self.buffer.push(event);
        }
        
        self.buffer_pos.fetch_add(1, Ordering::Relaxed);
        self.metrics.recorder_events_total.inc();
    }

    /// Flush buffer to disk (called asynchronously)
    pub fn flush(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let pos = self.buffer_pos.load(Ordering::Relaxed);
        
        if pos == 0 {
            return Ok(()); // Nothing to flush
        }

        // Open output file
        let path = format!("{}/recording_{}.jsonl", 
            self.config.recorder.output_path,
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_secs()
        );
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        // Write events as JSON lines
        for event in &self.buffer[..pos] {
            let json = format!(
                r#"{{"timestamp":{},"type":"{}","metadata":{{"source":"{}","dest":"{}"}}}}"#,
                event.timestamp,
                match event.event_type {
                    EventType::HttpRequest => "http_request",
                    EventType::HttpResponse => "http_response",
                    EventType::TcpConnection => "tcp_connection",
                    EventType::UdpPacket => "udp_packet",
                },
                event.metadata.source,
                event.metadata.destination
            );
            writeln!(file, "{}", json)?;
            
            let size = json.len() as u64;
            self.metrics.recorder_bytes_written.inc_by(size);
        }

        // Reset buffer
        self.buffer_pos.store(0, Ordering::Relaxed);
        
        Ok(())
    }
}

// Empty stub for non-recorder builds
#[cfg(not(feature = "recorder"))]
pub struct Recorder;

#[cfg(not(feature = "recorder"))]
impl Recorder {
    // This ensures recorder code is not compiled in prod builds
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "recorder")]
    use super::*;

    #[cfg(feature = "recorder")]
    #[test]
    fn test_recorder_creation() {
        let config = Arc::new(crate::config::Config::default());
        let metrics = crate::metrics::Metrics::new().unwrap();
        let recorder = Recorder::new(config, metrics);
        assert!(!recorder.is_enabled());
    }

    #[cfg(feature = "recorder")]
    #[test]
    fn test_recorder_enable_disable() {
        let config = Arc::new(crate::config::Config::default());
        let metrics = crate::metrics::Metrics::new().unwrap();
        let recorder = Recorder::new(config, metrics);
        
        assert!(!recorder.is_enabled());
        recorder.enable();
        assert!(recorder.is_enabled());
        recorder.disable();
        assert!(!recorder.is_enabled());
    }
}
