# Examples

## TCP Proxy Example

Forward TCP traffic from port 8080 to a PostgreSQL database on port 5432:

```bash
# Using environment variables
LISTEN_ADDR=0.0.0.0:8080 \
TARGET_ADDR=localhost:5432 \
PROXY_MODE=tcp \
./target/release/superpilot
```

Access metrics:
```bash
curl http://localhost:9090/metrics
```

## UDP Forwarder Example

Forward UDP traffic (e.g., for DNS or syslog):

```bash
LISTEN_ADDR=0.0.0.0:53 \
TARGET_ADDR=8.8.8.8:53 \
PROXY_MODE=udp \
./target/release/superpilot
```

## HTTP Proxy Example

Observe HTTP traffic with method, path, and status tracking:

```bash
LISTEN_ADDR=0.0.0.0:8080 \
TARGET_ADDR=backend:8081 \
PROXY_MODE=http \
./target/release/superpilot
```

## Configuration File Example

Create a `config.yaml`:

```yaml
listen_addr: "0.0.0.0:8080"
target_addr: "127.0.0.1:8081"
mode: tcp
metrics_addr: "0.0.0.0:9090"

tcp:
  buffer_size: 8192
  connection_timeout_secs: 300

safety:
  max_memory_mb: 256
  max_latency_ms: 100
  rate_limit_requests_per_sec: 10000
  sampling_rate: 1.0
```

Run with config:
```bash
CONFIG_FILE=config.yaml ./target/release/superpilot
```

## Docker Compose Example

```yaml
version: '3.8'

services:
  app:
    image: myapp:latest
    ports:
      - "8081:8080"

  sidecar:
    image: superpilot:latest
    environment:
      - LISTEN_ADDR=0.0.0.0:8080
      - TARGET_ADDR=app:8080
      - PROXY_MODE=tcp
    ports:
      - "8080:8080"  # Application traffic
      - "9090:9090"  # Metrics endpoint
    depends_on:
      - app
```

## Recorder Mode Example

**Warning:** Recorder mode includes MITM capabilities and payload capture. Use only in non-production environments.

```bash
# Build recorder binary
cargo build --release --features recorder

# Run with recording enabled
./target/release/superpilot-recorder
```

With recording configuration:

```yaml
listen_addr: "0.0.0.0:8080"
target_addr: "127.0.0.1:8081"
mode: http

recorder:
  enabled: true
  mode: metadata  # or 'payload' for full capture
  buffer_size: 1048576
  output_path: "/tmp/recordings"
  tls_mitm: false
```

## Kubernetes Sidecar Example

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: myapp-with-sidecar
spec:
  containers:
  - name: app
    image: myapp:latest
    ports:
    - containerPort: 8081
  
  - name: superpilot
    image: superpilot:latest
    ports:
    - containerPort: 8080  # Proxy port
    - containerPort: 9090  # Metrics port
    env:
    - name: LISTEN_ADDR
      value: "0.0.0.0:8080"
    - name: TARGET_ADDR
      value: "localhost:8081"
    - name: PROXY_MODE
      value: "tcp"
    resources:
      limits:
        memory: "256Mi"
        cpu: "500m"
```

## Prometheus Scraping Configuration

```yaml
scrape_configs:
  - job_name: 'superpilot'
    static_configs:
    - targets: ['localhost:9090']
    metrics_path: /metrics
    scrape_interval: 15s
```

## Performance Testing

Test with `wrk`:

```bash
# Start superpilot
LISTEN_ADDR=0.0.0.0:8080 \
TARGET_ADDR=localhost:8081 \
./target/release/superpilot &

# Run load test
wrk -t4 -c100 -d30s http://localhost:8080/

# Check metrics
curl http://localhost:9090/metrics | grep -E '(tcp_|http_)'
```

## Safety Features Demo

Test rate limiting:

```yaml
safety:
  rate_limit_requests_per_sec: 100  # Limit to 100 req/s
  sampling_rate: 0.1                # Sample 10% of traffic
```

Monitor safety metrics:
```bash
curl http://localhost:9090/metrics | grep safety_
```
