# BlackLake Deployment Guide
# Week 5: Production deployment and operations

## Overview

This guide covers deploying BlackLake in production environments using Docker Compose with multi-architecture support and comprehensive monitoring.

## Prerequisites

- Docker Engine 20.10+
- Docker Compose 2.0+
- Docker Buildx (for multi-arch builds)
- 8GB+ RAM
- 100GB+ disk space
- Network access to external services

## Quick Start

### 1. Clone Repository

```bash
git clone https://github.com/blacklake/blacklake.git
cd blacklake
```

### 2. Configure Environment

```bash
cp env.example .env
# Edit .env with your configuration
```

### 3. Start Development Environment

```bash
# Using justfile (recommended)
just dev

# Or using docker compose directly
docker compose --profile dev up -d --wait
```

### 4. Verify Deployment

```bash
# Check API health
curl http://localhost:8080/live

# Check UI
curl http://localhost:3000

# Check MinIO console
curl http://localhost:9001
```

## Production Deployment

### 1. Environment Configuration

Create a production `.env` file:

```bash
# Database
POSTGRES_PASSWORD=your-secure-password
POSTGRES_DB=blacklake

# MinIO
MINIO_ROOT_PASSWORD=your-secure-password

# Keycloak
KEYCLOAK_ADMIN_PASSWORD=your-secure-password

# API
RUST_LOG=info
API_PORT=8080

# UI
UI_PORT=3000

# Monitoring
GRAFANA_ADMIN_PASSWORD=your-secure-password
```

### 2. Start Production Stack

```bash
# Start production services
just up-prod

# Or manually
docker compose --profile prod up -d --wait
```

### 3. Initialize Services

```bash
# Run database migrations
just migrate

# Initialize MinIO buckets
docker compose exec minio mc mb minio/blacklake
docker compose exec minio mc mb minio/exports
docker compose exec minio mc mb minio/mlflow
```

### 4. Configure Keycloak

1. Access Keycloak admin console: http://localhost:8081
2. Login with admin credentials
3. Create realm for BlackLake
4. Create client for API
5. Create users and roles

## Multi-Architecture Builds

### Using Docker Bake

```bash
# Build all images locally
just bake

# Build and push to registry
just bake-push

# Build specific target
just bake-target api

# Build for specific platform
just bake-platform linux/arm64 api
```

### Manual Build

```bash
# Set up buildx
docker buildx create --name multiarch --use

# Build multi-arch
docker buildx build --platform linux/amd64,linux/arm64 -t blacklake/api:latest .

# Push to registry
docker buildx build --platform linux/amd64,linux/arm64 -t ghcr.io/blacklake/api:latest --push .
```

## Docker Compose Profiles

### Available Profiles

- `dev` (default): Development environment
- `prod`: Production environment
- `search-os`: OpenSearch integration
- `av`: Antivirus scanning
- `ui-dev`: UI development with hot reload
- `ml`: MLflow integration

### Profile Usage

```bash
# Development with OpenSearch
docker compose --profile dev --profile search-os up -d

# Production with antivirus
docker compose --profile prod --profile av up -d

# Development with MLflow
docker compose --profile dev --profile ml up -d
```

## Environment Variables

### Required Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `POSTGRES_PASSWORD` | Database password | `blacklake` |
| `MINIO_ROOT_PASSWORD` | MinIO admin password | `minioadmin` |
| `KEYCLOAK_ADMIN_PASSWORD` | Keycloak admin password | `admin` |
| `GRAFANA_ADMIN_PASSWORD` | Grafana admin password | `admin` |

### Optional Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `API_PORT` | API server port | `8080` |
| `UI_PORT` | UI server port | `3000` |
| `RUST_LOG` | Rust logging level | `info` |
| `REGISTRY` | Container registry | `ghcr.io` |
| `IMAGE_PREFIX` | Image prefix | `blacklake` |

## Monitoring and Observability

### Prometheus Metrics

- **API Metrics**: http://localhost:8080/metrics
- **Prometheus UI**: http://localhost:9090
- **Grafana Dashboards**: http://localhost:3001

### Health Checks

```bash
# API health
curl http://localhost:8080/live
curl http://localhost:8080/ready

# Database health
docker compose exec db pg_isready

# MinIO health
docker compose exec minio mc admin info minio
```

### Logging

```bash
# View all logs
just logs-all

# View specific service logs
just logs api

# View structured logs
just logs-structured
```

## Backup and Recovery

### Automated Backups

```bash
# Create backup
just backup

# Restore from backup
just restore backups/blacklake-20240115.tar.gz
```

### Manual Backup

```bash
# Database backup
docker compose exec -T db pg_dump -U blacklake blacklake > backup.sql

# MinIO backup
docker compose exec -T minio mc mirror minio/blacklake backups/minio/

# Configuration backup
cp docker-compose.yml backups/
cp .env backups/
```

## Security Considerations

### Network Security

- Use reverse proxy (nginx/traefik) for TLS termination
- Configure firewall rules
- Use VPN for admin access
- Enable rate limiting

### Container Security

- Run containers as non-root users
- Use distroless base images
- Scan images for vulnerabilities
- Keep base images updated

### Data Security

- Encrypt data at rest
- Use secure passwords
- Enable audit logging
- Regular security updates

## Scaling

### Horizontal Scaling

```bash
# Scale API instances
docker compose up -d --scale api=3

# Scale with load balancer
docker compose --profile prod --profile gateway up -d
```

### Vertical Scaling

- Increase container memory limits
- Add more CPU cores
- Optimize database settings
- Use SSD storage

## Troubleshooting

### Common Issues

#### Services Not Starting

```bash
# Check service status
just status

# View service logs
just logs api

# Check resource usage
just stats
```

#### Database Connection Issues

```bash
# Check database logs
just logs db

# Test database connection
docker compose exec db psql -U blacklake -d blacklake -c "SELECT 1;"
```

#### Storage Issues

```bash
# Check MinIO status
docker compose exec minio mc admin info minio

# Check disk usage
just disk-usage
```

### Performance Issues

```bash
# Run load tests
just load-test

# Check metrics
just metrics

# View resource usage
just stats
```

## Maintenance

### Regular Maintenance

```bash
# Update images
docker compose pull
docker compose up -d

# Clean up resources
just cleanup

# Run security scans
just scan
```

### Database Maintenance

```bash
# Run migrations
just migrate

# Backup database
just backup

# Analyze database
docker compose exec db psql -U blacklake -d blacklake -c "ANALYZE;"
```

## Support

### Getting Help

- Check logs: `just logs-all`
- View metrics: `just metrics`
- Run diagnostics: `just status`
- Check documentation: `docs/`

### Reporting Issues

1. Check existing issues on GitHub
2. Collect relevant logs and metrics
3. Create detailed issue report
4. Include environment information

## References

- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Docker Buildx Documentation](https://docs.docker.com/buildx/)
- [BlackLake Operations Guide](OPERATIONS.md)
- [BlackLake Images Guide](IMAGES.md)