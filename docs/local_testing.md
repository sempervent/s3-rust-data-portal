# Local Testing Guide

This guide provides comprehensive instructions for setting up and testing the BlackLake project locally using Docker and optimized Rust builds.

## 🚀 Quick Start

### One-Command Setup
```bash
# Complete setup in one command
just setup-all && just dev && just migrate
```

## 📋 Prerequisites

### Required Tools
```bash
# Install cargo-chef for optimal Docker builds
cargo install cargo-chef

# Install just (command runner)
brew install just

# Install Docker and Docker Compose
# Docker Desktop for Mac/Windows or Docker Engine for Linux
```

### Verify Installation
```bash
# Check tools are installed
cargo --version
just --version
docker --version
docker compose --version
```

## 🏗️ Building Services

### Build All Services (Optimized)
```bash
# Build all services with cargo-chef optimization
docker buildx bake all

# Or build individual services
docker buildx bake api-local
docker buildx bake cli-local
docker buildx bake ui-local
docker buildx bake gateway-local
```

### Build for Specific Platform
```bash
# Build for ARM64 (Apple Silicon)
docker buildx build --platform linux/arm64 -f Dockerfile.cli -t blacklake-cli:arm64 .

# Build for AMD64 (Intel)
docker buildx build --platform linux/amd64 -f Dockerfile.cli -t blacklake-cli:amd64 .
```

## 🚀 Starting Development Environment

### Start All Services
```bash
# Start all services with migrations
just dev

# Or manually with docker-compose
docker compose --profile dev up -d --wait
```

### Run Database Migrations
```bash
# Run database migrations
just migrate

# Or manually
docker compose run --rm migrations
```

## 🧪 Testing CLI Functionality

### Test CLI Commands
```bash
# Open CLI shell
just cli-shell
# Inside container: blacklake --help
# Inside container: blacklake init --help
# Inside container: blacklake put --help

# Run specific CLI commands
just cli-run "init --help"
just cli-run "put --help"
```

### Test CLI with Direct Build
```bash
# Test with optimized build
./test-cli-direct.sh

# Test with standard build
./test-cli-native.sh
```

## 🌐 Accessing Services

### Service URLs
- **API**: http://localhost:8080
- **UI**: http://localhost:3000
- **MinIO**: http://localhost:9001
- **PostgreSQL**: localhost:5432

### Service Health Checks
```bash
# Check API health
curl http://localhost:8080/health

# Check UI
curl http://localhost:3000

# Check MinIO
curl http://localhost:9001/minio/health/live
```

## 🔧 Development Commands

### View Logs
```bash
# View all logs
just logs

# View specific service logs
docker compose logs api
docker compose logs cli
docker compose logs db
```

### Stop Services
```bash
# Stop all services
just stop

# Or manually
docker compose down
```

### Clean Up
```bash
# Clean up containers and volumes
just clean

# Or manually
docker compose down -v
docker system prune -f
```

### Rebuild Services
```bash
# Rebuild specific service
docker buildx bake api-local --no-cache
docker buildx bake cli-local --no-cache

# Rebuild all services
docker buildx bake all --no-cache
```

## 🧪 Testing Workflows

### Test Complete CLI Workflow
```bash
# 1. Initialize a directory
just cli-run "init ./test-data --repo test-repo --ref main"

# 2. Upload files
just cli-run "put ./test-data --repo test-repo --ref main"

# 3. Search for files
just cli-run "search --query test"
```

### Test API Endpoints
```bash
# Test API endpoints
curl -X GET http://localhost:8080/v1/repos
curl -X GET http://localhost:8080/v1/repos/test-repo
curl -X GET http://localhost:8080/v1/repos/test-repo/tree/main
```

## 🐛 Troubleshooting

### Common Issues

#### Platform Mismatch (Apple Silicon)
```bash
# Error: GLIBC version not found
# Solution: Use ARM64 platform
docker buildx build --platform linux/arm64 -f Dockerfile.cli -t blacklake-cli:arm64 .
```

#### Database Connection Issues
```bash
# Check database is running
docker compose ps db

# Check database logs
docker compose logs db

# Restart database
docker compose restart db
```

#### Build Cache Issues
```bash
# Clear build cache
docker buildx prune -f

# Rebuild without cache
docker buildx bake all --no-cache
```

### Debug Commands
```bash
# Check running containers
docker compose ps

# Check container logs
docker compose logs -f api

# Execute commands in running container
docker compose exec api bash
docker compose exec cli bash
```

## 📊 Performance Optimization

### Build Optimization
- **cargo-chef**: Dependency caching for faster builds
- **Multi-stage builds**: Smaller final images
- **Layer caching**: Reuse unchanged layers
- **Multi-platform**: ARM64 + AMD64 support

### Runtime Optimization
- **Non-root user**: Security best practices
- **Minimal runtime**: Debian slim base images
- **Resource limits**: Configured in docker-compose.yml

## 🔄 Development Workflow

### Daily Development
```bash
# Start development environment
just dev

# Make code changes
# ... edit code ...

# Rebuild affected services
docker buildx bake api-local
docker buildx bake cli-local

# Test changes
just cli-run "init --help"
```

### Testing Changes
```bash
# Run tests
just test

# Run specific test
just cli-run "init ./test-data --dry-run"
```

## 📝 Additional Resources

### Documentation
- [Implementation Summary](IMPLEMENTATION_SUMMARY.md)
- [Migration Setup](MIGRATION_SETUP.md)
- [API Documentation](../api/)

### Useful Commands
```bash
# Show all available just commands
just --list

# Show Docker Compose services
docker compose config --services

# Show build targets
docker buildx bake --print
```

## 🎯 Next Steps

1. **Test CLI functionality** with the provided test scripts
2. **Explore the API** using the service URLs
3. **Run migrations** to set up the database schema
4. **Test the complete workflow** from init to upload

For more detailed information, see the main [documentation.md](documentation.md) and [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md).

