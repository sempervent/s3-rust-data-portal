# BlackLake Quick Start Guide

This guide will get you up and running with BlackLake using a simplified approach.

## 🚀 Step-by-Step Setup

### 1. Prerequisites
```bash
# Install required tools
cargo install cargo-chef
brew install just
```

### 2. Build All Services
```bash
# Build all Docker images first
just bake
```

### 3. Start Everything (Simplified)
```bash
# Start all services at once (simplified approach)
just setup-step-by-step
```

### 4. Alternative: Manual Step-by-Step
```bash
# Step 1: Start database and dependencies
docker compose -f docker-compose.simple.yml up -d db minio redis keycloak solr

# Step 2: Run migrations
just migrate

# Step 3: Start API and UI
docker compose -f docker-compose.simple.yml up -d api ui
```

### 5. Verify Everything is Working
```bash
# Check all services
docker compose -f docker-compose.simple.yml ps

# Test API
curl http://localhost:8080/health

# Test UI
open http://localhost:5173

# Test MinIO
open http://localhost:9001
```

## 🔧 Alternative: One-Command Setup

If you want to start everything at once (after building):

```bash
# Build first
just bake

# Start everything
just dev

# Run migrations
just migrate
```

## 🧪 Test CLI Functionality

```bash
# Test CLI commands
just cli-shell
# Inside container: blacklake --help
# Inside container: blacklake init --help
# Inside container: blacklake put --help
```

## 🌐 Service URLs

- **API**: http://localhost:8080
- **UI (Dev)**: http://localhost:5173
- **MinIO**: http://localhost:9001
- **Keycloak**: http://localhost:8081
- **PostgreSQL**: localhost:5432

## 🐛 Troubleshooting

### If UI doesn't start:
```bash
# Check if UI service is running
docker compose ps ui-dev

# Check UI logs
docker compose logs ui-dev

# Restart UI
docker compose restart ui-dev
```

### If API doesn't start:
```bash
# Check API logs
docker compose logs api

# Check if database is ready
docker compose logs db
```

### If migrations fail:
```bash
# Check database is running
docker compose ps db

# Run migrations manually
docker compose run --rm migrations
```

## 📋 Service Dependencies

The correct startup order is:
1. **Database** (PostgreSQL)
2. **Dependencies** (MinIO, Redis, Keycloak, Solr)
3. **Migrations** (Database schema)
4. **API** (Backend service)
5. **UI** (Frontend service)

## 🎯 Next Steps

Once everything is running:
1. **Test the API**: Visit http://localhost:8080/health
2. **Test the UI**: Visit http://localhost:5173
3. **Test CLI**: Run `just cli-shell`
4. **Create your first repository**: Use the CLI to init and upload data
