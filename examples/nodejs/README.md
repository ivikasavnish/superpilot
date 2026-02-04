# Node.js with Superpilot Sidecar

This example shows how to integrate Superpilot as a sidecar with your Node.js application for observability.

## Files

- `Dockerfile.dev` - Development Docker image for Node.js application
- `Dockerfile.prod` - Production Docker image for Node.js application  
- `docker-compose.yml` - Development environment with superpilot sidecar
- `docker-compose.prod.yml` - Production environment with superpilot sidecar

## Quick Start

### Development

1. Create your Node.js application with an `index.js` file that listens on port 8081
2. Add a `package.json` file with your dependencies
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

Create a simple Express `index.js`:

```javascript
const express = require('express');
const app = express();
const port = process.env.PORT || 8081;

app.get('/', (req, res) => {
  res.send('Hello from Node.js with Superpilot!');
});

app.listen(port, '0.0.0.0', () => {
  console.log(`Server running on port ${port}`);
});
```

Example `package.json`:

```json
{
  "name": "myapp",
  "version": "1.0.0",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "dev": "nodemon index.js"
  },
  "dependencies": {
    "express": "^4.18.2"
  },
  "devDependencies": {
    "nodemon": "^3.0.1"
  }
}
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

- Hot reload with nodemon and volume mounting
- Direct access to application logs
- Node modules cached in anonymous volume
- Easy debugging

## Production Features

- Minimal Alpine-based image
- Non-root user for security
- Production dependencies only (via npm ci)
- Production-optimized superpilot binary
- Automatic restart on failure
- Health monitoring via metrics endpoint
