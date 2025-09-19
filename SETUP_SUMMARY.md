# 🚀 BlackLake Fast Setup - Complete Guide

## ⚡ **TL;DR - Fastest Setup**

```bash
# One command to build everything and start the stack
./quick-setup.sh
```

## 🎯 **What We Built**

### **1. Optimized Build System**
- **`docker-bake-simple.hcl`**: Clean, working Docker Buildx configuration
- **`quick-setup.sh`**: One-command setup script
- **`build-optimized.sh`**: Advanced build script with caching strategies
- **Enhanced `justfile`**: Added fast setup commands

### **2. DRY & Smart Implementation**
- **Build Groups**: `dev`, `core`, `local`, `all` for different use cases
- **Parallel Builds**: Maximum parallelism for faster builds
- **Smart Caching**: Local, GitHub Actions, and registry cache support
- **Single Platform**: AMD64-only builds for development speed

### **3. Multiple Setup Methods**

#### **Method 1: One Command (Easiest)**
```bash
./quick-setup.sh
```

#### **Method 2: Using Just (Recommended)**
```bash
just setup-all    # Complete setup
just setup-dev    # Development only
just setup-prod   # Production
```

#### **Method 3: Optimized Build Script**
```bash
./build-optimized.sh dev     # Development
./build-optimized.sh core    # Core services
./build-optimized.sh all     # Everything
./build-optimized.sh fast    # Single platform
```

#### **Method 4: Direct Docker Commands**
```bash
# Build all images
docker buildx bake -f docker-bake-simple.hcl local

# Start development stack
docker compose --profile dev up -d --wait
```

## 📋 **Build Groups Reference**

| Group | Targets | Use Case | Build Time |
|-------|---------|----------|------------|
| `dev` | api-local, ui-local | Development essentials | ~1-2 min |
| `core` | api-local, ui-local, gateway-local | Core services | ~2-3 min |
| `local` | api-local, ui-local, gateway-local | Development | ~2-3 min |
| `all` | All services | Complete build | ~3-5 min |

## 🔧 **Build Optimization Features**

### **Smart Caching**
```bash
# Local cache (fastest for development)
./build-optimized.sh dev local

# GitHub Actions cache (for CI/CD)
./build-optimized.sh dev gha

# Registry cache (for production)
./build-optimized.sh dev registry
```

### **Parallel Execution**
```bash
# Build multiple targets in parallel
docker buildx bake -f docker-bake-simple.hcl --parallel all
```

### **Single Platform (Fastest)**
```bash
# Build for AMD64 only (faster than multi-arch)
./build-optimized.sh fast
```

## 🎯 **Recommended Workflows**

### **First Time Setup**
```bash
# Option 1: One command (easiest)
./quick-setup.sh

# Option 2: Optimized (fastest)
./build-optimized.sh dev

# Option 3: Manual (most control)
docker buildx bake -f docker-bake-simple.hcl local
docker compose --profile dev up -d --wait
```

### **Daily Development**
```bash
# Quick start
just setup-dev

# With specific features
docker compose --profile dev --profile search-os up -d
```

### **Production Deployment**
```bash
# Build production images
./build-optimized.sh prod

# Or using just
just setup-prod
```

## 🌐 **Access Points After Setup**

- **API**: http://localhost:8080
- **UI**: http://localhost:8080 (served by gateway)
- **Grafana**: http://localhost:3000 (admin/admin)
- **Solr**: http://localhost:8983
- **MinIO**: http://localhost:9001 (minioadmin/minioadmin)
- **Keycloak**: http://localhost:8081 (admin/admin)

## 🔍 **Troubleshooting**

### **Common Issues**

1. **Docker not running**
   ```bash
   # Check Docker status
   docker info
   ```

2. **Build failures**
   ```bash
   # Check build logs
   docker buildx bake -f docker-bake-simple.hcl --progress=plain dev
   ```

3. **Service startup issues**
   ```bash
   # Check service logs
   docker compose logs -f api
   ```

### **Performance Tips**

1. **Use single platform for development**
   ```bash
   ./build-optimized.sh fast
   ```

2. **Enable BuildKit**
   ```bash
   export DOCKER_BUILDKIT=1
   export COMPOSE_DOCKER_CLI_BUILD=1
   ```

3. **Use local cache**
   ```bash
   ./build-optimized.sh dev local
   ```

## 📚 **Documentation**

- **FAST_SETUP.md**: Comprehensive build strategies and optimization tips
- **README.md**: Updated with fast setup section
- **justfile**: Enhanced with setup commands
- **docker-bake-simple.hcl**: Clean, working build configuration

## 🎉 **Success Metrics**

- **One-Command Setup**: ✅ `./quick-setup.sh`
- **Multiple Methods**: ✅ 4 different setup approaches
- **Build Optimization**: ✅ Parallel builds, caching, single platform
- **DRY Implementation**: ✅ Reusable build groups and functions
- **Smart Caching**: ✅ Local, GHA, and registry cache support
- **Production Ready**: ✅ All environments supported

---

## 🚀 **Ready to Go!**

The BlackLake project now has a **comprehensive, optimized, and DRY build system** that supports:

- ⚡ **Fast setup** with one command
- 🔧 **Multiple methods** for different use cases
- 📦 **Smart caching** for faster builds
- 🎯 **Build groups** for different environments
- 🚀 **Parallel execution** for maximum speed
- 🏭 **Production ready** with all features

**Choose your preferred method and start building!** 🎯
