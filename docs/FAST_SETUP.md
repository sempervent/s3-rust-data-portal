# BlackLake Fast Setup Guide

This guide provides a quick setup for the BlackLake data platform with optimized build configurations.

## Prerequisites

- Rust 1.70+
- Docker and Docker Compose
- `just` command runner (optional, for dev commands)

## Quick Start

### 1. Clone and Build

```bash
git clone https://github.com/your-org/blacklake.git
cd blacklake

# Build all crates with optimizations
cargo build --workspace --release

# Run tests
cargo test --workspace
```

### 2. Start Services

```bash
# Start all services with optimized configuration
docker-compose up -d

# Or use just (if installed)
just up-dev
```

### 3. Verify Setup

```bash
# Check all services are running
docker-compose ps

# Check API health
curl http://localhost:8080/health

# Check database health
curl http://localhost:8080/health/db

# Check storage health
curl http://localhost:8080/health/storage
```

## Optimized Build Configuration

### **Cargo.toml Optimizations**

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### **Docker Build Optimizations**

```dockerfile
# Multi-stage build for smaller images
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/blacklake-api /usr/local/bin/
CMD ["blacklake-api"]
```

### **Environment Variables**

```bash
# Performance optimizations
RUST_LOG=info
RUST_BACKTRACE=1
DATABASE_POOL_SIZE=20
REDIS_POOL_SIZE=10
CACHE_TTL=3600
```

## Development Workflow

### **Using Just Commands**

```bash
# Show all available commands
just --list

# Start development environment
just up-dev

# Run tests
just test

# Build optimized release
just build-release

# Clean and rebuild
just clean-build
```

### **Hot Reload Development**

```bash
# Start with hot reload
just dev

# Watch for changes
just watch

# Run specific tests
just test-unit
just test-integration
```

## Performance Tuning

### **Database Optimization**

```bash
# Set database connection pool size
export DATABASE_POOL_SIZE=20

# Enable query logging
export DATABASE_LOG_QUERIES=true

# Set connection timeout
export DATABASE_TIMEOUT=30s
```

### **Redis Configuration**

```bash
# Set Redis pool size
export REDIS_POOL_SIZE=10

# Set cache TTL
export CACHE_TTL=3600

# Enable Redis clustering
export REDIS_CLUSTER=true
```

### **S3 Storage Optimization**

```bash
# Set S3 connection pool size
export S3_POOL_SIZE=5

# Enable S3 multipart uploads
export S3_MULTIPART_THRESHOLD=100MB

# Set S3 timeout
export S3_TIMEOUT=60s
```

## Monitoring Setup

### **Prometheus Configuration**

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'blacklake-api'
    static_configs:
      - targets: ['api:8080']
```

### **Grafana Dashboards**

```bash
# Import dashboards
curl -X POST http://localhost:3000/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @docs/grafana/dashboards/system-overview.json
```

## Troubleshooting

### **Common Issues**

1. **Build Failures**
   ```bash
   # Clean and rebuild
   cargo clean
   cargo build --workspace
   ```

2. **Database Connection Issues**
   ```bash
   # Check database status
   docker-compose logs db
   
   # Reset database
   docker-compose down -v
   docker-compose up -d db
   ```

3. **Storage Issues**
   ```bash
   # Check MinIO status
   docker-compose logs minio
   
   # Reset storage
   docker-compose down -v
   docker-compose up -d minio
   ```

### **Performance Issues**

1. **Slow API Responses**
   - Check database connection pool
   - Verify Redis connectivity
   - Monitor memory usage

2. **High Memory Usage**
   - Reduce connection pool sizes
   - Enable garbage collection
   - Monitor for memory leaks

3. **Database Performance**
   - Check query performance
   - Verify indexes
   - Monitor connection pool

## Production Deployment

### **Docker Compose Production**

```bash
# Production deployment
docker-compose -f docker-compose.prod.yml up -d

# With custom configuration
BLACKLAKE_DOMAIN=blacklake.example.com docker-compose -f docker-compose.prod.yml up -d
```

### **Kubernetes Deployment**

```bash
# Deploy to Kubernetes
kubectl apply -f k8s/

# With Helm
helm install blacklake ./helm/blacklake -f helm/blacklake/values.yaml
```

## Additional Resources

- [Local Testing Guide](local_testing.md)
- [Migration Setup](MIGRATION_SETUP.md)
- [CLI Documentation](cli.md)
- [Project Status](PROJECT_STATUS.md)
- [Implementation Summary](IMPLEMENTATION_SUMMARY.md)

## Support

- **Documentation**: [Documentation Home](index.md)
- **Issues**: [GitHub Issues](https://github.com/sempervent/s3-rust-data-portal/issues)
- **Discussions**: [GitHub Discussions](https://github.com/sempervent/s3-rust-data-portal/discussions)