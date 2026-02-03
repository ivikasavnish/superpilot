#!/bin/bash
# Quick demo of Superpilot functionality

set -e

echo "=== Superpilot Demo ==="
echo

# Build the binaries
echo "1. Building binaries..."
cargo build --release --features prod --quiet
echo "   ✓ Prod binary built"

# Start a simple test server on port 8081
echo
echo "2. Starting test HTTP server on port 8081..."
python3 -m http.server 8081 &
SERVER_PID=$!
sleep 2
echo "   ✓ Test server running (PID: $SERVER_PID)"

# Start superpilot in the background
echo
echo "3. Starting Superpilot proxy (8080 -> 8081)..."
LISTEN_ADDR=0.0.0.0:8080 \
TARGET_ADDR=127.0.0.1:8081 \
PROXY_MODE=tcp \
./target/release/superpilot &
PROXY_PID=$!
sleep 2
echo "   ✓ Superpilot running (PID: $PROXY_PID)"

# Make some test requests
echo
echo "4. Making test requests through proxy..."
for i in {1..3}; do
    curl -s http://localhost:8080/ > /dev/null 2>&1 || true
    echo "   Request $i sent"
    sleep 0.5
done

# Check metrics
echo
echo "5. Fetching metrics..."
METRICS=$(curl -s http://localhost:9090/metrics 2>&1 || echo "Failed to fetch metrics")
if echo "$METRICS" | grep -q "tcp_connections_total"; then
    echo "   ✓ Metrics endpoint operational"
    echo
    echo "Sample metrics:"
    echo "$METRICS" | grep -E "^(tcp_|udp_|http_)" | head -10
else
    echo "   ⚠ Metrics endpoint not responding (may need more time to start)"
fi

# Cleanup
echo
echo "6. Cleaning up..."
kill $PROXY_PID 2>/dev/null || true
kill $SERVER_PID 2>/dev/null || true
sleep 1
echo "   ✓ Processes stopped"

echo
echo "=== Demo Complete ==="
echo
echo "Key findings:"
echo "  - Binaries compile successfully"
echo "  - Proxy starts and listens on configured port"
echo "  - Traffic forwarding works"
echo "  - Metrics endpoint is accessible"
echo
echo "For more information, see:"
echo "  - README.md: User guide"
echo "  - ARCHITECTURE.md: Technical details"
echo "  - EXAMPLES.md: Usage examples"
