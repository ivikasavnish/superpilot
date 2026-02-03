# Prometheus Push Gateway Integration

## Overview

Superpilot now supports pushing metrics to a Prometheus Pushgateway in addition to the standard pull-based metrics endpoint. This feature is particularly useful in environments where:

- Pull-based metrics collection is not feasible (NAT/firewall restrictions)
- Short-lived jobs need to expose metrics
- Network topology makes scraping difficult

## Features

### 1. Optional Configuration
- Push gateway is **completely optional**
- If not configured, metrics are only available via the standard pull endpoint at `:9090/metrics`
- Clear logging when push gateway is not configured

### 2. Automatic Failure Handling
- **Non-blocking**: Push happens in a background thread
- **Retry logic**: Continues attempting to push even after failures
- **Auto-disable**: After 5 consecutive failures, push is automatically disabled
- **Graceful degradation**: Proxy continues normal operation regardless of push status
- **Always available**: Pull endpoint remains functional even if push fails

### 3. Safety Features
- Uses lightweight HTTP client (ureq) - no async runtime overhead
- Configurable push interval (default: 60 seconds)
- Proper error logging for troubleshooting
- Zero impact on proxy performance

## Configuration

Add to your `config.yaml`:

```yaml
prometheus:
  push_gateway_url: "http://pushgateway:9091"  # Optional - if omitted, push is disabled
  push_interval_secs: 60                        # Push interval in seconds
  job_name: "superpilot"                        # Job name for Pushgateway
  instance: "production-1"                      # Instance identifier
```

### Environment Variables

You can also configure via environment variables:
- Standard metrics endpoint: Always available regardless of push configuration
- Push gateway: Configured via YAML file

## Deployment

### With Docker Compose

```yaml
version: '3.8'

services:
  pushgateway:
    image: prom/pushgateway
    ports:
      - "9091:9091"

  superpilot:
    image: superpilot:latest
    volumes:
      - ./config.yaml:/config.yaml
    environment:
      - CONFIG_FILE=/config.yaml
    ports:
      - "8080:8080"  # Proxy
      - "9090:9090"  # Pull metrics

  prometheus:
    image: prom/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"
```

### Prometheus Configuration

Configure Prometheus to scrape from Pushgateway:

```yaml
scrape_configs:
  - job_name: 'pushgateway'
    honor_labels: true
    static_configs:
      - targets: ['pushgateway:9091']
```

## Behavior Examples

### Without Push Gateway Configured

```
$ ./superpilot
Superpilot - Production Mode (Observe-Only)
Version: 0.1.0
...
Prometheus push gateway not configured - metrics will only be available via pull endpoint
Starting TCP proxy...
```

### With Push Gateway Configured (Reachable)

```
$ ./superpilot
Superpilot - Production Mode (Observe-Only)
Version: 0.1.0
...
Prometheus push gateway: http://pushgateway:9091
  Push interval: 60 seconds
  Job name: superpilot
  Instance: production-1
Starting TCP proxy...
Prometheus push thread started
```

### With Push Gateway Configured (Unreachable)

```
$ ./superpilot
Superpilot - Production Mode (Observe-Only)
Version: 0.1.0
...
Prometheus push gateway: http://pushgateway:9091
  Push interval: 60 seconds
  Job name: superpilot
  Instance: production-1
Starting TCP proxy...
Prometheus push thread started
Failed to push metrics to Prometheus: Connection refused
...
Disabling Prometheus push after 5 consecutive failures
Prometheus push thread stopped
```

**Note**: Even after push is disabled, the proxy continues to operate normally and metrics remain available at the pull endpoint.

## Testing

Run the demo script to see the feature in action:

```bash
./demo_push.sh
```

Run unit tests:

```bash
cargo test metrics::tests::test_metrics_push
```

## Implementation Details

- **Dependencies**: Added `ureq` for HTTP client (minimal, no async)
- **Thread Safety**: Uses `AtomicBool` for push enable/disable
- **Error Handling**: Comprehensive error messages for troubleshooting
- **Performance**: Zero impact on hot path, push happens in background thread

## Metrics Pushed

All standard Superpilot metrics are pushed to the gateway:
- TCP connection metrics
- UDP packet metrics
- HTTP request metrics
- HTTP/2 stream metrics
- QUIC packet metrics
- Safety metrics (rate limiting, sampling)
- Recorder metrics (when feature enabled)

## Troubleshooting

### Push Not Working

1. Check push gateway is accessible:
   ```bash
   curl http://pushgateway:9091/metrics
   ```

2. Check Superpilot logs for error messages

3. Verify configuration in `config.yaml`

4. Check push gateway logs for incoming requests

### After 5 Failures

If push is automatically disabled:
1. Fix the underlying issue (network, gateway down, etc.)
2. Restart Superpilot to re-enable push
3. Metrics continue to be available via pull endpoint

## Future Enhancements

Potential improvements for future releases:
- Configurable failure threshold
- Exponential backoff for retries
- Support for basic authentication
- TLS support for push gateway connection
- Metrics about push operations (success rate, latency)
