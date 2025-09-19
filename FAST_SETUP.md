# 🚀 BlackLake Fast Setup Guide

## ⚡ **One-Command Setup**

```bash
# Build everything and start the full stack
just setup-all
```

## 🎯 **Smart Build Strategy**

### **1. Build All Images (Multi-Arch + Local)**
```bash
# Build all images locally with caching (FASTEST)
just bake

# Build specific components only
just bake-target api
just bake-target ui
just bake-target gateway
```

### **2. Start Development Stack**
```bash
# Start full development environment
just up-dev

# Start with specific profiles
just up-profiles "dev,search-os"  # Include Solr
just up-profiles "dev,av"         # Include ClamAV
just up-profiles "dev,ml"         # Include MLflow
```

### **3. Build Rust Artifacts**
```bash
# Build all Rust crates
just build

# Build in release mode (optimized)
just build-release

# Build specific crate
just build-crate api
just build-crate core
```

## 🔧 **Advanced Build Commands**

### **Multi-Architecture Builds**
```bash
# Build for specific platform only (faster)
just bake-platform "linux/amd64" api
just bake-platform "linux/arm64" ui

# Build and push to registry
just bake-push
```

### **Development Workflow**
```bash
# Hot reload development
just up-dev && just logs api

# Run tests
just test

# Lint and format
just lint
just fmt
```

## 📦 **Build Groups (DRY Approach)**

The `docker-bake.hcl` defines smart build groups:

```bash
# Core services only
docker buildx bake -f docker-bake-simple.hcl core

# Development essentials
docker buildx bake -f docker-bake-simple.hcl dev

# Production stack
docker buildx bake -f docker-bake-simple.hcl prod

# Everything
docker buildx bake -f docker-bake-simple.hcl all

# Local development (fastest)
docker buildx bake -f docker-bake-simple.hcl local
```

## ⚡ **Optimized Build Order**

### **Parallel Build Strategy**
```bash
# Build core services in parallel
docker buildx bake -f docker-bake-simple.hcl --parallel core &

# Build additional services
docker buildx bake -f docker-bake-simple.hcl --parallel observability &
docker buildx bake -f docker-bake-simple.hcl --parallel ml &

# Wait for all builds
wait
```

### **Dependency-Aware Building**
```bash
# 1. Build base images first (shared layers)
docker buildx bake -f docker-bake-simple.hcl api ui

# 2. Build dependent services
docker buildx bake -f docker-bake-simple.hcl gateway jobrunner

# 3. Build optional services
docker buildx bake -f docker-bake-simple.hcl otel-collector mlflow
```

## 🎯 **Smart Caching Strategy**

### **Leverage BuildKit Cache**
```bash
# Use GitHub Actions cache (if available)
docker buildx bake -f docker-bake-simple.hcl --cache-from type=gha all

# Use local cache
docker buildx bake -f docker-bake-simple.hcl --cache-from type=local all

# Use registry cache
docker buildx bake -f docker-bake-simple.hcl --cache-from type=registry,ref=ghcr.io/blacklake/api:cache all
```

### **Layer Optimization**
```bash
# Build with maximum cache utilization
docker buildx bake -f docker-bake-simple.hcl --cache-to type=local,dest=/tmp/.buildx-cache all
```

## 🚀 **Production Build Pipeline**

### **Complete Production Build**
```bash
# 1. Build all images with security attestations
docker buildx bake -f docker-bake-simple.hcl secure

# 2. Build remaining services
docker buildx bake -f docker-bake-simple.hcl otel-collector mlflow

# 3. Push to registry
docker buildx bake --push all
```

### **Multi-Stage Build Optimization**
```bash
# Build with specific build args for optimization
docker buildx bake \
  --set *.output=type=docker \
  --set *.build-arg:BUILDKIT_INLINE_CACHE=1 \
  --set *.cache-from=type=local,src=/tmp/.buildx-cache \
  all
```

## 🔧 **Environment-Specific Builds**

### **Development (Fast Iteration)**
```bash
# Quick local build
just bake && just up-dev

# With hot reload
just up-profiles "dev" && just logs api
```

### **Staging (Production-like)**
```bash
# Build production images locally
docker buildx bake -f docker-bake-simple.hcl prod

# Start production stack
just up-prod
```

### **Production (Multi-Arch + Security)**
```bash
# Build with security attestations
docker buildx bake -f docker-bake-simple.hcl secure

# Build remaining services
docker buildx bake -f docker-bake-simple.hcl otel-collector mlflow

# Push to registry
docker buildx bake --push all
```

## 📊 **Build Performance Tips**

### **1. Use BuildKit Features**
```bash
# Enable BuildKit
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# Use BuildKit cache mounts
docker buildx bake -f docker-bake-simple.hcl --cache-from type=gha all
```

### **2. Parallel Builds**
```bash
# Build multiple targets in parallel
docker buildx bake -f docker-bake-simple.hcl --parallel all
```

### **3. Platform-Specific Builds**
```bash
# Build for your platform only (faster)
docker buildx bake -f docker-bake-simple.hcl --set *.platform=linux/amd64 all
```

### **4. Layer Caching**
```bash
# Use registry cache for faster builds
docker buildx bake \
  --set *.output=type=docker \
  --set *.cache-from=type=registry,ref=ghcr.io/blacklake/api:cache \
  all
```

## 🎯 **One-Liner Commands**

### **Complete Setup**
```bash
# Build everything and start development
just bake && just up-dev && just logs api
```

### **Production Ready**
```bash
# Build and start production stack
docker buildx bake -f docker-bake-simple.hcl prod && just up-prod
```

### **Quick Development**
```bash
# Start with core services only
just up-profiles "dev" && just logs api
```

### **Full Stack with All Features**
```bash
# Build and start everything
just bake && just up-profiles "dev,search-os,av,ml" && just logs-all
```

## 🔍 **Build Verification**

### **Check Build Status**
```bash
# List all built images
docker images | grep blacklake

# Check build cache
docker buildx du

# Verify multi-arch builds
docker buildx imagetools inspect ghcr.io/blacklake/api:latest
```

### **Test Builds**
```bash
# Test API build
docker run --rm blacklake-api:local --version

# Test UI build
docker run --rm blacklake-ui:local --version
```

## 📋 **Build Groups Reference**

| Group | Targets | Use Case |
|-------|---------|----------|
| `local` | api-local, ui-local, gateway-local | Development |
| `dev` | api, ui | Development essentials |
| `core` | api, ui, gateway | Core services |
| `prod` | api, ui, gateway | Production stack |
| `all` | All services | Complete build |
| `secure` | api-secure, ui-secure | Security-focused |
| `observability` | otel-collector | Monitoring |
| `ml` | mlflow | ML features |

## 🎯 **Recommended Workflow**

### **First Time Setup**
```bash
# 1. Build all images
just bake

# 2. Start development stack
just up-dev

# 3. Check logs
just logs api
```

### **Daily Development**
```bash
# Quick start
just up-dev && just logs api

# With specific features
just up-profiles "dev,search-os" && just logs solr
```

### **Production Deployment**
```bash
# Build production images
docker buildx bake -f docker-bake-simple.hcl prod

# Deploy
just up-prod
```

---

## 🚀 **TL;DR - Fastest Setup**

```bash
# One command to rule them all
just bake && just up-dev
```

This builds all images locally and starts the development stack. The `docker-bake.hcl` configuration ensures optimal caching and parallel builds for maximum speed.
