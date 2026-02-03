use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::config::Config;
use crate::metrics::Metrics;

/// Safety guard for preventing resource exhaustion
/// Implements rate limiting, sampling, and auto-disable
pub struct SafetyGuard {
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    
    // Rate limiting (atomic counters)
    connections_this_second: AtomicU64,
    packets_this_second: AtomicU64,
    last_reset: AtomicU64, // Timestamp in seconds
    
    // Sampling
    sample_counter: AtomicU64,
    
    // Auto-disable state
    recorder_disabled: AtomicBool,
    disable_timestamp: AtomicU64,
}

impl SafetyGuard {
    pub fn new(config: Arc<Config>, metrics: Arc<Metrics>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        SafetyGuard {
            config,
            metrics,
            connections_this_second: AtomicU64::new(0),
            packets_this_second: AtomicU64::new(0),
            last_reset: AtomicU64::new(now),
            sample_counter: AtomicU64::new(0),
            recorder_disabled: AtomicBool::new(false),
            disable_timestamp: AtomicU64::new(0),
        }
    }

    /// Check if connection should be allowed (rate limiting)
    pub fn should_allow_connection(&self) -> bool {
        self.reset_counters_if_needed();

        let count = self.connections_this_second.fetch_add(1, Ordering::Relaxed);
        let limit = self.config.safety.rate_limit_requests_per_sec;

        if count >= limit {
            return false;
        }

        // Apply sampling
        self.should_sample()
    }

    /// Check if packet should be allowed (rate limiting for UDP)
    pub fn should_allow_packet(&self) -> bool {
        self.reset_counters_if_needed();

        let count = self.packets_this_second.fetch_add(1, Ordering::Relaxed);
        let limit = self.config.safety.rate_limit_requests_per_sec;

        if count >= limit {
            return false;
        }

        // Apply sampling
        self.should_sample()
    }

    /// Sampling decision (no locks, atomic counter)
    fn should_sample(&self) -> bool {
        let sample_rate = self.config.safety.sampling_rate;
        if sample_rate >= 1.0 {
            return true; // No sampling
        }

        let counter = self.sample_counter.fetch_add(1, Ordering::Relaxed);
        let threshold = (1.0 / sample_rate) as u64;
        
        if counter % threshold == 0 {
            true
        } else {
            self.metrics.safety_sampled_out.inc();
            false
        }
    }

    /// Reset rate limit counters every second
    fn reset_counters_if_needed(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let last = self.last_reset.load(Ordering::Relaxed);
        
        if now > last {
            // Try to update (only one thread succeeds)
            if self.last_reset.compare_exchange(
                last,
                now,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ).is_ok() {
                // Reset counters
                self.connections_this_second.store(0, Ordering::Relaxed);
                self.packets_this_second.store(0, Ordering::Relaxed);
            }
        }
    }

    /// Check if recorder should be disabled (latency spike)
    #[cfg(feature = "recorder")]
    pub fn check_latency(&self, latency: std::time::Duration) -> bool {
        let max_latency = std::time::Duration::from_millis(self.config.safety.max_latency_ms);
        
        if latency > max_latency && !self.recorder_disabled.load(Ordering::Relaxed) {
            self.disable_recorder("latency_spike");
            return false;
        }
        
        true
    }

    /// Check if recorder should be disabled (memory pressure)
    #[cfg(feature = "recorder")]
    pub fn check_memory(&self) -> bool {
        // Simple memory check using /proc/self/statm on Linux
        // In production, use more sophisticated memory tracking
        let _max_memory_bytes = self.config.safety.max_memory_mb * 1024 * 1024;
        
        // Placeholder: would need actual memory tracking
        // For now, always allow
        true
    }

    /// Disable recorder and record metric
    #[cfg(feature = "recorder")]
    pub fn disable_recorder(&self, reason: &str) {
        if self.recorder_disabled.compare_exchange(
            false,
            true,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ).is_ok() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            self.disable_timestamp.store(now, Ordering::Relaxed);
            self.metrics.recorder_auto_disabled.inc();
            
            eprintln!("Recorder auto-disabled due to: {}", reason);
        }
    }

    /// Check if recorder is disabled
    #[cfg(feature = "recorder")]
    pub fn is_recorder_disabled(&self) -> bool {
        self.recorder_disabled.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_guard_creation() {
        let config = Arc::new(Config::default());
        let metrics = Metrics::new().unwrap();
        let guard = SafetyGuard::new(config, metrics);
        assert!(true); // Basic creation test
    }

    #[test]
    fn test_rate_limiting() {
        let mut config = Config::default();
        config.safety.rate_limit_requests_per_sec = 10;
        config.safety.sampling_rate = 1.0; // No sampling
        
        let config = Arc::new(config);
        let metrics = Metrics::new().unwrap();
        let guard = SafetyGuard::new(config, metrics);

        // First 10 should be allowed
        for _ in 0..10 {
            assert!(guard.should_allow_connection());
        }

        // 11th should be denied
        assert!(!guard.should_allow_connection());
    }

    #[test]
    fn test_sampling() {
        let mut config = Config::default();
        config.safety.rate_limit_requests_per_sec = 100000; // High limit
        config.safety.sampling_rate = 0.5; // 50% sampling
        
        let config = Arc::new(config);
        let metrics = Metrics::new().unwrap();
        let guard = SafetyGuard::new(config, metrics);

        let mut allowed = 0;
        let total = 100;
        
        for _ in 0..total {
            if guard.should_allow_connection() {
                allowed += 1;
            }
        }

        // Should be roughly 50% allowed (with some variance)
        assert!(allowed > 30 && allowed < 70);
    }
}
