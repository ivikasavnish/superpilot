# Golang with Superpilot Sidecar

This example shows how to integrate Superpilot as a sidecar with your Golang application for observability.

## Files

- `Dockerfile.dev` - Development Docker image for Golang application
- `Dockerfile.prod` - Production Docker image for Golang application  
- `docker-compose.yml` - Development environment with superpilot sidecar
- `docker-compose.prod.yml` - Production environment with superpilot sidecar

## Quick Start

### Development

1. Create your Golang application with a `main.go` file that listens on port 8081
2. Add a `go.mod` file with your dependencies
3. Run with Docker Compose:

```bash
docker-compose up
```

Your application will be:
- Running on port 8081 (internal)
- Accessible via superpilot proxy on port 8080
- Metrics available at http://localhost:9090/metrics

### Production

```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Example Application

Create a simple `main.go`:

```go
package main

import (
    "fmt"
    "log"
    "net/http"
)

func handler(w http.ResponseWriter, r *http.Request) {
    fmt.Fprintf(w, "Hello from Golang with Superpilot!")
}

func main() {
    http.HandleFunc("/", handler)
    log.Println("Server starting on port 8081...")
    log.Fatal(http.ListenAndServe(":8081", nil))
}
```

Example `go.mod`:

```go
module myapp

go 1.21
```

## Accessing Your Application

- **Application**: http://localhost:8080 (proxied through superpilot)
- **Direct Access**: http://localhost:8081 (bypass superpilot)
- **Metrics**: http://localhost:9090/metrics

## Customization

### Change Proxy Mode

Edit the `docker-compose.yml` file and change the `PROXY_MODE` environment variable:

- `tcp` - TCP proxy (default for non-HTTP services)
- `http` - HTTP proxy with method, path, and status tracking
- `udp` - UDP forwarder

### Add Configuration

Create a `config.yaml` file and mount it in the superpilot container:

```yaml
volumes:
  - ./config.yaml:/app/config.yaml
environment:
  - CONFIG_FILE=/app/config.yaml
```

## Development Features

- Hot reload with volume mounting
- Direct access to application logs
- Easy debugging with port forwarding

## Production Features

- Multi-stage build for minimal image size
- Production-optimized superpilot binary
- Automatic restart on failure
- Health monitoring via metrics endpoint
