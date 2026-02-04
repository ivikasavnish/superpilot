# Python with Superpilot Sidecar

This example shows how to integrate Superpilot as a sidecar with your Python application for observability.

## Files

- `Dockerfile.dev` - Development Docker image for Python application
- `Dockerfile.prod` - Production Docker image for Python application  
- `docker-compose.yml` - Development environment with superpilot sidecar
- `docker-compose.prod.yml` - Production environment with superpilot sidecar

## Quick Start

### Development

1. Create your Python application with an `app.py` file that listens on port 8081
2. Add a `requirements.txt` file with your dependencies
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

Create a simple Flask `app.py`:

```python
from flask import Flask

app = Flask(__name__)

@app.route('/')
def hello():
    return 'Hello from Python with Superpilot!'

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8081, debug=True)
```

Example `requirements.txt`:

```
flask==3.0.0
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
- PYTHONUNBUFFERED for immediate log output
- Flask debug mode enabled
- Direct access to application logs

## Production Features

- Minimal image size with slim base
- Non-root user for security
- Production-optimized superpilot binary
- Automatic restart on failure
- Health monitoring via metrics endpoint
