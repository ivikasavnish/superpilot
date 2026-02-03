# Superpilot - Production-Safe Docker Sidecar

A high-performance, low-latency Rust-based Docker sidecar for black-box observability and optional traffic recording.

## Features

- **Dual-mode operation**: Production (observe-only) and Recorder (MITM + capture)
- **L4 TCP Proxy**: Transparent forwarding for databases and services
- **UDP Forwarder**: Stateless best-effort packet forwarding
- **HTTP/1 Support**: Method, path, status, and latency observation
- **HTTP/2 & HTTP/3**: Stream tracking and RTT metrics
- **Production Safety**: Rate limiting, sampling, auto-disable on resource pressure
- **Prometheus Metrics**: Native /metrics endpoint with low cardinality

## Architecture

Built with Rust 2021 for maximum performance and safety:

- No eBPF or kernel modules
- No async runtime on hot path
- No blocking on recording
- Fixed buffers, atomics, bounded queues
- Feature flags separate prod and recorder builds

## Quick Start

### Production Mode (Default)

```bash
cargo build --release --features prod
./target/release/superpilot
```

### Recorder Mode (Optional)

```bash
cargo build --release --features recorder
./target/release/superpilot
```

### Docker

```bash
# Build image
docker build -t superpilot .

# Run prod mode
docker run -p 8080:8080 -p 9090:9090 superpilot

# Run recorder mode
docker run -p 8080:8080 -p 9090:9090 superpilot /app/superpilot-recorder
```

## Configuration

### Environment Variables

- `LISTEN_ADDR`: Address to listen on (default: 0.0.0.0:8080)
- `TARGET_ADDR`: Target service address (default: 127.0.0.1:8081)
- `PROXY_MODE`: tcp, udp, or http (default: tcp)
- `CONFIG_FILE`: Path to YAML config file

### Configuration File

See `config.yaml` for example configuration.

## Metrics

Access Prometheus metrics at `http://localhost:9090/metrics`

### Prometheus Push Gateway (Optional)

Superpilot supports pushing metrics to a Prometheus Pushgateway for environments where the pull model is not suitable.

Configure the push gateway in your `config.yaml`:

```yaml
prometheus:
  push_gateway_url: "http://pushgateway:9091"
  push_interval_secs: 60
  job_name: "superpilot"
  instance: "localhost"
```

If `push_gateway_url` is not configured, Superpilot will log a message and metrics will only be available via the pull endpoint.

**Safety Features:**
- If the push gateway is unreachable or returns errors, Superpilot will log the failure
- After 5 consecutive failures, the push feature will be automatically disabled
- The proxy continues to function normally even if push fails
- Metrics are always available via the pull endpoint regardless of push status

### Key Metrics

- `tcp_connections_total`: Total TCP connections
- `tcp_bytes_sent_total`: Total bytes sent
- `tcp_bytes_received_total`: Total bytes received
- `http_requests_total`: HTTP requests by method and status
- `http_request_duration_seconds`: Request latency histogram
- `safety_rate_limited_total`: Rate-limited requests
- `recorder_events_total`: Recorded events (recorder mode only)

## Testing

```bash
# Run unit tests
cargo test

# Run specific test
cargo test test_tcp_proxy_creation

# Test with recorder feature
cargo test --features recorder
```

## Performance

Designed for production use:

- Hot path uses fixed buffers (no heap allocations)
- Atomic operations for lock-free metrics
- Per-connection threads (no async overhead)
- Zero-copy forwarding where possible
- Minimal dependencies

## Safety

Multiple layers of protection:

- Rate limiting (connections/packets per second)
- Sampling (configurable rate)
- Size limits (headers, body, packets)
- Auto-disable on:
  - Latency spikes
  - Memory pressure
  - Disk fill (recorder mode)

## License

See LICENSE file for details.