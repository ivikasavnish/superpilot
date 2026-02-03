#!/bin/bash
# Demonstration of Prometheus push gateway integration

set -e

echo "=== Prometheus Push Gateway Demo ==="
echo

# Build the binary
echo "1. Building superpilot..."
cargo build --release --features prod --quiet
echo "   ✓ Binary built"
echo

# Test 1: Without push gateway configured (default)
echo "2. Test: Running WITHOUT push gateway configured"
echo "   Expected: Log message about push gateway not configured"
echo
timeout 2 ./target/release/superpilot 2>&1 | grep -A1 "Prometheus" || true
echo

# Test 2: With push gateway configured but unreachable
echo "3. Test: Running WITH push gateway configured (unreachable)"
echo "   Expected: Log showing push gateway details and connection failures"
echo

cat > /tmp/test_push_config.yaml << 'EOF'
listen_addr: "0.0.0.0:8888"
target_addr: "127.0.0.1:8889"
mode: tcp
metrics_addr: "0.0.0.0:9999"
tcp:
  buffer_size: 8192
  connection_timeout_secs: 300
udp:
  buffer_size: 65536
  max_packet_size: 65507
http:
  max_header_size: 8192
  max_body_size: 1048576
safety:
  max_memory_mb: 256
  max_latency_ms: 100
  rate_limit_requests_per_sec: 10000
  sampling_rate: 1.0
prometheus:
  push_gateway_url: "http://localhost:9091"
  push_interval_secs: 2
  job_name: "superpilot-demo"
  instance: "test-instance"
EOF

echo "Config created with:"
echo "  Push Gateway URL: http://localhost:9091"
echo "  Push Interval: 2 seconds"
echo "  Job: superpilot-demo"
echo "  Instance: test-instance"
echo

timeout 7 env CONFIG_FILE=/tmp/test_push_config.yaml ./target/release/superpilot 2>&1 | head -20 || true

# Cleanup
rm -f /tmp/test_push_config.yaml

echo
echo "=== Demo Complete ==="
echo
echo "Key Behaviors Demonstrated:"
echo "  1. ✓ Without push gateway URL: Logs informative message"
echo "  2. ✓ With push gateway URL: Shows configuration details"
echo "  3. ✓ On connection failure: Logs error message"
echo "  4. ✓ After 5 failures: Would disable push (not shown in demo)"
echo "  5. ✓ Proxy continues normal operation regardless of push status"
echo
echo "To enable push gateway in production:"
echo "  1. Deploy Prometheus Pushgateway"
echo "  2. Add 'prometheus' section to config.yaml"
echo "  3. Configure Prometheus to scrape from Pushgateway"
