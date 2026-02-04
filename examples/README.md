# Superpilot Language Stack Examples

This directory contains Docker configurations for integrating Superpilot as a sidecar with various application stacks. Each example includes both development and production configurations.

## Available Examples

- **[Golang](./golang/)** - Go applications with superpilot sidecar
- **[Python](./python/)** - Python/Flask applications with superpilot sidecar
- **[Node.js](./nodejs/)** - Node.js/Express applications with superpilot sidecar
- **[Ruby](./ruby/)** - Ruby/Sinatra applications with superpilot sidecar
- **[Rails](./rails/)** - Ruby on Rails applications with superpilot sidecar and PostgreSQL

## What is Superpilot?

Superpilot is a high-performance, production-safe Docker sidecar for black-box observability. It provides:

- **Dual-mode operation**: Production (observe-only) and Recorder (MITM + capture)
- **L4 TCP Proxy**: Transparent forwarding for databases and services
- **UDP Forwarder**: Stateless best-effort packet forwarding
- **HTTP Support**: Method, path, status, and latency observation
- **Production Safety**: Rate limiting, sampling, auto-disable on resource pressure
- **Prometheus Metrics**: Native /metrics endpoint with low cardinality

## How It Works

Each example follows the sidecar pattern:

```
┌─────────────────┐         ┌──────────────────┐
│   Your App      │         │   Superpilot     │
│   (port 8081)   │◄────────┤   (port 8080)    │◄──── External Traffic
│                 │         │   + Metrics      │
└─────────────────┘         │   (port 9090)    │
                            └──────────────────┘
```

Traffic flows through Superpilot, which:
1. Observes and collects metrics
2. Forwards traffic to your application
3. Exposes Prometheus metrics at `/metrics`

## Common Structure

Each language example includes:

```
language/
├── Dockerfile.dev          # Development image
├── Dockerfile.prod         # Production image  
├── docker-compose.yml      # Dev environment with sidecar
├── docker-compose.prod.yml # Prod environment with sidecar
└── README.md              # Language-specific instructions
```

## Quick Start

Choose your stack and navigate to its directory:

```bash
cd examples/[golang|python|nodejs|ruby|rails]
```

### Development Mode

```bash
# Start your app with superpilot sidecar
docker-compose up

# Access your application
curl http://localhost:8080

# View metrics
curl http://localhost:9090/metrics
```

### Production Mode

```bash
# Start production environment
docker-compose -f docker-compose.prod.yml up -d

# Access your application
curl http://localhost:8080

# View metrics
curl http://localhost:9090/metrics
```

## Key Features by Mode

### Development Mode

- Hot reload with volume mounting
- Debug logging enabled
- Direct access to application logs
- Fast iteration cycles
- Observer-only superpilot binary

### Production Mode

- Multi-stage builds for minimal images
- Production-optimized binaries
- Non-root users for security
- Automatic restart policies
- Health monitoring
- Production superpilot binary with safety features

## Metrics Available

Superpilot exposes these key metrics:

- `tcp_connections_total` - Total TCP connections
- `tcp_bytes_sent_total` - Total bytes sent
- `tcp_bytes_received_total` - Total bytes received
- `http_requests_total` - HTTP requests by method and status
- `http_request_duration_seconds` - Request latency histogram
- `safety_rate_limited_total` - Rate-limited requests

## Proxy Modes

Superpilot supports three proxy modes (configure via `PROXY_MODE` env var):

- **tcp** - TCP proxy for databases and TCP services
- **http** - HTTP proxy with method, path, and status tracking
- **udp** - UDP forwarder for DNS, syslog, etc.

## Configuration

### Environment Variables

All examples support these Superpilot environment variables:

- `LISTEN_ADDR` - Address superpilot listens on (default: 0.0.0.0:8080)
- `TARGET_ADDR` - Your application address (default: app:8081)
- `PROXY_MODE` - tcp, udp, or http (default: http)
- `CONFIG_FILE` - Path to YAML config file (optional)

### Configuration File

For advanced configuration, mount a `config.yaml`:

```yaml
listen_addr: "0.0.0.0:8080"
target_addr: "app:8081"
mode: http
metrics_addr: "0.0.0.0:9090"

safety:
  max_memory_mb: 256
  max_latency_ms: 100
  rate_limit_requests_per_sec: 10000
  sampling_rate: 1.0

prometheus:
  push_gateway_url: "http://pushgateway:9091"
  push_interval_secs: 60
  job_name: "superpilot"
```

## Customizing Examples

Each example can be customized for your needs:

1. **Change ports**: Edit the `ports` section in docker-compose.yml
2. **Add volumes**: Mount your code or config files
3. **Environment variables**: Add app-specific env vars
4. **Database**: Rails example includes PostgreSQL; adapt for your DB
5. **Proxy mode**: Change `PROXY_MODE` based on your protocol

## Integration Patterns

### Pattern 1: Direct Integration (Examples)

Use the example docker-compose files directly for simple applications.

### Pattern 2: Existing Docker Compose

Add superpilot to your existing docker-compose.yml:

```yaml
services:
  your-app:
    # Your existing app config
    
  superpilot:
    image: superpilot:latest
    environment:
      - LISTEN_ADDR=0.0.0.0:8080
      - TARGET_ADDR=your-app:8081
      - PROXY_MODE=http
    ports:
      - "8080:8080"
      - "9090:9090"
```

### Pattern 3: Kubernetes Sidecar

Deploy as a sidecar in Kubernetes:

```yaml
spec:
  containers:
  - name: app
    image: your-app:latest
  - name: superpilot
    image: superpilot:latest
    env:
    - name: TARGET_ADDR
      value: "localhost:8081"
```

## Building Superpilot

To build the superpilot image:

```bash
cd ../..  # Back to repo root
docker build -t superpilot .
```

The Dockerfile creates both prod and recorder binaries:
- `/app/superpilot` or `/app/superpilot-prod` - Production mode (observe-only)
- `/app/superpilot-recorder` - Recorder mode (MITM + capture)

## Troubleshooting

### Port Conflicts

If ports 8080 or 9090 are in use, change them in docker-compose.yml:

```yaml
ports:
  - "3000:8080"  # External:Internal
```

### Cannot Connect to Superpilot

Ensure `TARGET_ADDR` points to your app's service name and port:

```yaml
environment:
  - TARGET_ADDR=app:8081  # service_name:port
```

### Metrics Not Available

Check if superpilot is running and port 9090 is exposed:

```bash
docker-compose ps
curl http://localhost:9090/metrics
```

### High Latency

Superpilot adds minimal latency (<1ms) but check:
- Rate limiting settings
- Resource constraints (CPU/memory)
- Network configuration

## Further Documentation

- [Main README](../../README.md) - Superpilot overview
- [Examples](../../EXAMPLES.md) - Additional usage examples
- [Architecture](../../ARCHITECTURE.md) - System design details

## Support

For issues or questions:
- Check language-specific READMEs in each directory
- Review the main repository documentation
- Open an issue on GitHub
