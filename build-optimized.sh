#!/bin/bash
# BlackLake Optimized Build Script
# DRY, smart, and fast implementation for all images and artifacts

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REGISTRY=${REGISTRY:-"ghcr.io"}
IMAGE_PREFIX=${IMAGE_PREFIX:-"blacklake"}
VERSION=${VERSION:-"latest"}
PLATFORMS=${PLATFORMS:-"linux/amd64,linux/arm64"}
CACHE_DIR="/tmp/.buildx-cache"

# Build groups (from docker-bake.hcl)
CORE_TARGETS="api ui gateway"
DEV_TARGETS="api ui"
PROD_TARGETS="api ui gateway"
ALL_TARGETS="api ui gateway jobrunner otel-collector mlflow"

# Functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v docker > /dev/null 2>&1; then
        log_error "Docker is not installed"
        exit 1
    fi
    
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker is not running"
        exit 1
    fi
    
    if ! command -v docker buildx > /dev/null 2>&1; then
        log_error "Docker Buildx is not available"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Setup buildx builder
setup_buildx() {
    log_info "Setting up Docker Buildx..."
    
    # Create buildx builder if it doesn't exist
    if ! docker buildx ls | grep -q "blacklake-builder"; then
        docker buildx create --name blacklake-builder --use
        log_success "Created buildx builder: blacklake-builder"
    else
        docker buildx use blacklake-builder
        log_success "Using existing buildx builder: blacklake-builder"
    fi
    
    # Bootstrap the builder
    docker buildx inspect --bootstrap
}

# Build with smart caching
build_with_cache() {
    local targets=$1
    local cache_strategy=$2
    
    log_info "Building targets: $targets with $cache_strategy caching"
    
    case $cache_strategy in
        "local")
            docker buildx bake \
                --set "*.output=type=docker" \
                --set "*.cache-from=type=local,src=$CACHE_DIR" \
                --set "*.cache-to=type=local,dest=$CACHE_DIR-new,mode=max" \
                $targets
            ;;
        "gha")
            docker buildx bake \
                --set "*.output=type=docker" \
                --set "*.cache-from=type=gha,scope=$IMAGE_PREFIX-$REGISTRY" \
                --set "*.cache-to=type=gha,mode=max,scope=$IMAGE_PREFIX-$REGISTRY" \
                $targets
            ;;
        "registry")
            docker buildx bake \
                --set "*.output=type=docker" \
                --set "*.cache-from=type=registry,ref=$REGISTRY/$IMAGE_PREFIX/cache:latest" \
                --set "*.cache-to=type=registry,ref=$REGISTRY/$IMAGE_PREFIX/cache:latest,mode=max" \
                $targets
            ;;
        "none")
            docker buildx bake --set "*.output=type=docker" $targets
            ;;
    esac
    
    # Rotate cache if using local caching
    if [ "$cache_strategy" = "local" ] && [ -d "$CACHE_DIR-new" ]; then
        rm -rf "$CACHE_DIR"
        mv "$CACHE_DIR-new" "$CACHE_DIR"
    fi
}

# Build Rust artifacts
build_rust() {
    log_info "Building Rust artifacts..."
    
    if command -v cargo > /dev/null 2>&1; then
        cargo build --release
        log_success "Rust artifacts built successfully"
    else
        log_warning "Cargo not available, skipping Rust build"
    fi
}

# Parallel build strategy
build_parallel() {
    local targets=$1
    local cache_strategy=$2
    
    log_info "Building in parallel: $targets"
    
    # Split targets and build in parallel
    IFS=' ' read -ra TARGET_ARRAY <<< "$targets"
    
    for target in "${TARGET_ARRAY[@]}"; do
        (
            log_info "Building $target..."
            build_with_cache "$target" "$cache_strategy"
            log_success "$target built successfully"
        ) &
    done
    
    # Wait for all builds to complete
    wait
    log_success "All parallel builds completed"
}

# Smart build order
build_smart() {
    local profile=$1
    local cache_strategy=${2:-"local"}
    
    log_info "Smart build for profile: $profile"
    
    case $profile in
        "dev")
            log_info "Building development essentials..."
            build_parallel "$DEV_TARGETS" "$cache_strategy"
            ;;
        "core")
            log_info "Building core services..."
            build_parallel "$CORE_TARGETS" "$cache_strategy"
            ;;
        "prod")
            log_info "Building production stack..."
            build_parallel "$PROD_TARGETS" "$cache_strategy"
            ;;
        "all")
            log_info "Building all services..."
            # Build in dependency order for better caching
            build_with_cache "$CORE_TARGETS" "$cache_strategy"
            build_parallel "jobrunner otel-collector mlflow" "$cache_strategy"
            ;;
        "fast")
            log_info "Fast build (single platform)..."
            docker buildx bake \
                --set "*.output=type=docker" \
                --set "*.platform=linux/amd64" \
                --set "*.cache-from=type=local,src=$CACHE_DIR" \
                $ALL_TARGETS
            ;;
    esac
}

# Start services
start_services() {
    local profile=$1
    
    log_info "Starting services with profile: $profile"
    
    case $profile in
        "dev")
            docker compose --profile dev up -d --wait
            ;;
        "prod")
            docker compose --profile prod up -d --wait
            ;;
        "all")
            docker compose --profile dev --profile search-os --profile av --profile ml up -d --wait
            ;;
    esac
    
    log_success "Services started successfully"
}

# Verify build
verify_build() {
    log_info "Verifying build..."
    
    # Check if images exist
    local images=("blacklake-api:local" "blacklake-ui:local" "blacklake-gateway:local")
    
    for image in "${images[@]}"; do
        if docker image inspect "$image" > /dev/null 2>&1; then
            log_success "Image $image exists"
        else
            log_warning "Image $image not found"
        fi
    done
    
    # Check service health
    if docker compose ps | grep -q "Up"; then
        log_success "Services are running"
    else
        log_warning "No services are running"
    fi
}

# Cleanup
cleanup() {
    log_info "Cleaning up..."
    docker system prune -f
    log_success "Cleanup completed"
}

# Main function
main() {
    local command=${1:-"dev"}
    local cache_strategy=${2:-"local"}
    
    echo "🚀 BlackLake Optimized Build"
    echo "============================"
    echo "Command: $command"
    echo "Cache Strategy: $cache_strategy"
    echo ""
    
    check_prerequisites
    setup_buildx
    
    case $command in
        "dev")
            build_smart "dev" "$cache_strategy"
            start_services "dev"
            ;;
        "core")
            build_smart "core" "$cache_strategy"
            start_services "dev"
            ;;
        "prod")
            build_smart "prod" "$cache_strategy"
            start_services "prod"
            ;;
        "all")
            build_smart "all" "$cache_strategy"
            start_services "all"
            ;;
        "fast")
            build_smart "fast" "$cache_strategy"
            start_services "dev"
            ;;
        "rust")
            build_rust
            ;;
        "verify")
            verify_build
            ;;
        "cleanup")
            cleanup
            ;;
        "help")
            echo "Usage: $0 [command] [cache_strategy]"
            echo ""
            echo "Commands:"
            echo "  dev      - Build and start development stack (default)"
            echo "  core     - Build core services only"
            echo "  prod     - Build and start production stack"
            echo "  all      - Build all services"
            echo "  fast     - Fast build (single platform)"
            echo "  rust     - Build Rust artifacts only"
            echo "  verify   - Verify build and services"
            echo "  cleanup  - Clean up Docker resources"
            echo "  help     - Show this help"
            echo ""
            echo "Cache Strategies:"
            echo "  local    - Use local cache (default)"
            echo "  gha      - Use GitHub Actions cache"
            echo "  registry - Use registry cache"
            echo "  none     - No caching"
            echo ""
            echo "Examples:"
            echo "  $0 dev local"
            echo "  $0 all gha"
            echo "  $0 fast none"
            ;;
        *)
            log_error "Unknown command: $command"
            echo "Use '$0 help' for usage information"
            exit 1
            ;;
    esac
    
    verify_build
    
    echo ""
    log_success "Build completed successfully!"
    echo "🌐 API: http://localhost:8080"
    echo "📊 Grafana: http://localhost:3000"
    echo "🔍 Solr: http://localhost:8983"
}

# Run main function with all arguments
main "$@"
