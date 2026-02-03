# Ruby with Superpilot Sidecar

This example shows how to integrate Superpilot as a sidecar with your Ruby application for observability.

## Files

- `Dockerfile.dev` - Development Docker image for Ruby application
- `Dockerfile.prod` - Production Docker image for Ruby application  
- `docker-compose.yml` - Development environment with superpilot sidecar
- `docker-compose.prod.yml` - Production environment with superpilot sidecar

## Quick Start

### Development

1. Create your Ruby application with an `app.rb` file that listens on port 8081
2. Add a `Gemfile` with your dependencies
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

Create a simple Sinatra `app.rb`:

```ruby
require 'sinatra'

set :bind, '0.0.0.0'
set :port, 8081

get '/' do
  'Hello from Ruby with Superpilot!'
end
```

Example `Gemfile`:

```ruby
source 'https://rubygems.org'

gem 'sinatra', '~> 3.1'
gem 'puma', '~> 6.4'
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
- Bundle cache for faster gem installation
- Direct access to application logs
- Easy debugging

## Production Features

- Minimal Alpine-based image
- Non-root user for security
- Production gems only (deployment mode)
- Production-optimized superpilot binary
- Automatic restart on failure
- Health monitoring via metrics endpoint
