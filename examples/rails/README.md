# Rails with Superpilot Sidecar

This example shows how to integrate Superpilot as a sidecar with your Rails application for observability.

## Files

- `Dockerfile.dev` - Development Docker image for Rails application
- `Dockerfile.prod` - Production Docker image for Rails application  
- `docker-compose.yml` - Development environment with superpilot sidecar and PostgreSQL
- `docker-compose.prod.yml` - Production environment with superpilot sidecar and PostgreSQL

## Quick Start

### Development

1. Place your Rails application in this directory
2. Ensure your `config/database.yml` uses the `DATABASE_URL` environment variable
3. Run with Docker Compose:

```bash
docker-compose up
```

Your application will be:
- Running on port 8081 (internal)
- Accessible via superpilot proxy on port 8080
- Metrics available at http://localhost:9090/metrics
- Connected to PostgreSQL database

### Production

Set required environment variables in a `.env` file:

```env
DATABASE_PASSWORD=your_secure_password
RAILS_MASTER_KEY=your_master_key
SECRET_KEY_BASE=your_secret_key_base
```

Then run:

```bash
docker-compose -f docker-compose.prod.yml up -d
```

## Database Configuration

Example `config/database.yml`:

```yaml
default: &default
  adapter: postgresql
  encoding: unicode
  pool: <%= ENV.fetch("RAILS_MAX_THREADS") { 5 } %>
  url: <%= ENV['DATABASE_URL'] %>

development:
  <<: *default

production:
  <<: *default
```

## Initial Setup

Run database migrations:

```bash
# Development
docker-compose run app bundle exec rails db:create db:migrate

# Production
docker-compose -f docker-compose.prod.yml run app bundle exec rails db:create db:migrate
```

## Accessing Your Application

- **Application**: http://localhost:8080 (proxied through superpilot)
- **Direct Access**: http://localhost:8081 (bypass superpilot)
- **Metrics**: http://localhost:9090/metrics
- **Database**: localhost:5432 (PostgreSQL)

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

### Use Different Database

Replace the `db` service in `docker-compose.yml` with your preferred database (MySQL, SQLite, etc.).

## Development Features

- Hot reload with volume mounting
- Bundle cache for faster gem installation
- PostgreSQL database included
- Direct access to application logs
- Rails development mode with debugging

## Production Features

- Multi-stage build for minimal image size
- Assets precompilation
- Non-root user for security
- Production gems only (deployment mode)
- Production-optimized superpilot binary
- Automatic restart on failure
- Health monitoring via metrics endpoint
- Persistent database storage

## Running Rails Commands

Development:
```bash
docker-compose run app bundle exec rails console
docker-compose run app bundle exec rails db:migrate
docker-compose run app bundle exec rails routes
```

Production:
```bash
docker-compose -f docker-compose.prod.yml run app bundle exec rails console -e production
docker-compose -f docker-compose.prod.yml run app bundle exec rails db:migrate
```

## Troubleshooting

### Port Already in Use

If port 8080 or 8081 is already in use, edit the `ports` section in `docker-compose.yml`:

```yaml
ports:
  - "3000:8080"  # Change external port
```

### Database Connection Issues

Ensure the `DATABASE_URL` environment variable is correctly set and the database service is running.

### Asset Precompilation Fails

Make sure all required Node.js packages are installed and the `RAILS_MASTER_KEY` is set.
