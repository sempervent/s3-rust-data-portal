# BlackLake Multi-Architecture Images Guide
# Week 5: Container image strategy and optimization

## Overview

BlackLake uses Docker Buildx and docker-bake.hcl to build multi-architecture container images supporting both AMD64 and ARM64 platforms. This guide covers the image strategy, build process, and optimization techniques.

## Image Architecture

### Supported Platforms

- **linux/amd64**: Intel/AMD x86_64 processors
- **linux/arm64**: ARM 64-bit processors (Apple Silicon, ARM servers)

### Image Components

#### Core Images
- **api**: BlackLake REST API server
- **ui**: React-based web interface
- **gateway**: Nginx reverse proxy
- **jobrunner**: Background job processor

#### Supporting Images
- **otel-collector**: OpenTelemetry data collection
- **mlflow**: ML experiment tracking (optional)

## Build Configuration

### docker-bake.hcl

The build configuration is defined in `docker-bake.hcl`:

```hcl
# Common variables
variable "REGISTRY" {
  default = "ghcr.io"
}

variable "IMAGE_PREFIX" {
  default = "blacklake"
}

variable "VERSION" {
  default = "latest"
}

# Common platforms
variable "PLATFORMS" {
  default = ["linux/amd64", "linux/arm64"]
}
```

### Build Targets

#### API Image
```hcl
target "api" {
  dockerfile = "Dockerfile.api"
  context = "."
  platforms = PLATFORMS
  tags = [
    "${REGISTRY}/${IMAGE_PREFIX}/api:${VERSION}",
    "${REGISTRY}/${IMAGE_PREFIX}/api:latest"
  ]
}
```

#### UI Image
```hcl
target "ui" {
  dockerfile = "Dockerfile.ui"
  context = "./ui"
  platforms = PLATFORMS
  tags = [
    "${REGISTRY}/${IMAGE_PREFIX}/ui:${VERSION}",
    "${REGISTRY}/${IMAGE_PREFIX}/ui:latest"
  ]
}
```

## Dockerfile Strategies

### Multi-Stage Builds

All images use multi-stage builds for optimization:

#### API Dockerfile
```dockerfile
# Planner stage
FROM rust:1.75-slim as planner
WORKDIR /app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder stage
FROM rust:1.75-slim as builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin api

# Runtime stage
FROM debian:bookworm-slim as runtime
RUN apt-get update && apt-get install -y ca-certificates libssl3 libpq5
COPY --from=builder /app/target/release/api /app/api
USER 1001
EXPOSE 8080
CMD ["./api"]
```

#### UI Dockerfile
```dockerfile
# Builder stage
FROM node:20-alpine as builder
RUN npm install -g pnpm
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN pnpm fetch
COPY . .
RUN pnpm install --frozen-lockfile
RUN pnpm build

# Runtime stage
FROM nginx:1.25-alpine as runtime
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
USER 1001
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### Security Best Practices

#### Non-Root Users
```dockerfile
# Create non-root user
RUN groupadd -r blacklake && useradd -r -g blacklake blacklake
USER blacklake
```

#### Minimal Base Images
- **API**: `debian:bookworm-slim` (minimal Debian)
- **UI**: `nginx:1.25-alpine` (Alpine Linux)
- **Gateway**: `nginx:1.25-alpine`

#### Security Scanning
```dockerfile
# Add security labels
LABEL org.opencontainers.image.title="BlackLake API"
LABEL org.opencontainers.image.description="BlackLake Data Artifact Management API"
LABEL org.opencontainers.image.vendor="BlackLake"
LABEL org.opencontainers.image.licenses="MIT"
```

## Build Process

### Local Development

#### Build Single Architecture
```bash
# Build for current platform
docker build -t blacklake-api:local .

# Build for specific platform
docker build --platform linux/amd64 -t blacklake-api:amd64 .
```

#### Build Multi-Architecture
```bash
# Build all targets locally
just bake

# Build specific target
just bake-target api

# Build for specific platform
just bake-platform linux/arm64 api
```

### CI/CD Pipeline

#### GitHub Actions
```yaml
- name: Set up Docker Buildx
  uses: docker/setup-buildx-action@v3

- name: Build and push
  uses: docker/build-push-action@v5
  with:
    context: .
    platforms: linux/amd64,linux/arm64
    push: true
    tags: ghcr.io/blacklake/api:latest
    cache-from: type=gha,scope=api
    cache-to: type=gha,mode=max,scope=api
```

#### Build Commands
```bash
# Build and push all images
just bake-push

# Build specific target
docker buildx bake --push api

# Build with custom registry
REGISTRY=my-registry.com IMAGE_PREFIX=myorg docker buildx bake --push all
```

## Caching Strategy

### Build Cache

#### Registry Cache
```hcl
function "cache_config" {
  returns = {
    "cache-from" = [
      "type=gha,scope=${IMAGE_PREFIX}-${REGISTRY}",
      "type=local,src=/tmp/.buildx-cache"
    ]
    "cache-to" = [
      "type=gha,mode=max,scope=${IMAGE_PREFIX}-${REGISTRY}",
      "type=local,dest=/tmp/.buildx-cache-new,mode=max"
    ]
  }
}
```

#### Layer Caching
```dockerfile
# Copy dependency files first
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy source code last
COPY src/ ./src/
RUN cargo build --release
```

### Runtime Cache

#### Package Manager Cache
```dockerfile
# Use pnpm fetch for better caching
COPY package.json pnpm-lock.yaml ./
RUN pnpm fetch
COPY . .
RUN pnpm install --frozen-lockfile
```

#### Build Cache
```dockerfile
# Use cargo-chef for Rust builds
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
```

## Image Optimization

### Size Optimization

#### Multi-Stage Builds
- Separate build and runtime stages
- Remove build dependencies in runtime
- Use minimal base images

#### Layer Optimization
```dockerfile
# Combine RUN commands
RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Use .dockerignore
echo "target/" >> .dockerignore
echo "node_modules/" >> .dockerignore
```

#### Compression
```dockerfile
# Use distroless images for production
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/api /api
ENTRYPOINT ["/api"]
```

### Performance Optimization

#### Build Performance
- Use BuildKit features
- Parallel builds
- Cache optimization
- Incremental builds

#### Runtime Performance
- Optimized base images
- Minimal attack surface
- Fast startup times
- Efficient resource usage

## Registry Management

### Image Tagging

#### Semantic Versioning
```bash
# Release tags
ghcr.io/blacklake/api:v1.2.3
ghcr.io/blacklake/api:v1.2
ghcr.io/blacklake/api:v1
ghcr.io/blacklake/api:latest
```

#### Development Tags
```bash
# Branch tags
ghcr.io/blacklake/api:main
ghcr.io/blacklake/api:develop
ghcr.io/blacklake/api:feature-branch

# Commit tags
ghcr.io/blacklake/api:sha-abc123
ghcr.io/blacklake/api:pr-123
```

### Registry Configuration

#### GitHub Container Registry
```yaml
# .github/workflows/build.yml
env:
  REGISTRY: ghcr.io
  IMAGE_PREFIX: blacklake
```

#### Custom Registry
```bash
# Configure custom registry
export REGISTRY=my-registry.com
export IMAGE_PREFIX=myorg
docker buildx bake --push all
```

## Security and Compliance

### Image Security

#### Vulnerability Scanning
```bash
# Scan with Trivy
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image blacklake/api:latest

# Scan with Grype
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  anchore/grype blacklake/api:latest
```

#### SBOM Generation
```bash
# Generate SBOM
docker buildx build --sbom=generator=image --push .
```

#### Provenance
```bash
# Generate provenance
docker buildx build --provenance=mode=max --push .
```

### Compliance

#### OCI Compliance
- Follow OCI image specification
- Use standard labels
- Implement proper metadata

#### Security Standards
- Non-root users
- Minimal base images
- Regular updates
- Vulnerability scanning

## Monitoring and Observability

### Image Metrics

#### Build Metrics
- Build time
- Image size
- Layer count
- Cache hit rate

#### Runtime Metrics
- Startup time
- Memory usage
- CPU usage
- Network performance

### Health Checks

#### Container Health
```dockerfile
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/live || exit 1
```

#### Application Health
```rust
// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}
```

## Troubleshooting

### Build Issues

#### Platform-Specific Builds
```bash
# Check available platforms
docker buildx ls

# Create new builder
docker buildx create --name multiarch --use

# Inspect builder
docker buildx inspect multiarch
```

#### Cache Issues
```bash
# Clear build cache
docker buildx prune

# Clear registry cache
docker buildx prune --filter type=registry
```

### Runtime Issues

#### Architecture Mismatch
```bash
# Check image architecture
docker inspect blacklake/api:latest | jq '.[0].Architecture'

# Run on specific platform
docker run --platform linux/amd64 blacklake/api:latest
```

#### Performance Issues
```bash
# Profile container performance
docker stats blacklake-api

# Check resource usage
docker exec blacklake-api top
```

## Best Practices

### Development

1. **Use multi-stage builds** for smaller images
2. **Cache dependencies** to speed up builds
3. **Use .dockerignore** to exclude unnecessary files
4. **Test on multiple platforms** before release

### Production

1. **Use distroless images** for security
2. **Implement health checks** for monitoring
3. **Use semantic versioning** for tags
4. **Scan for vulnerabilities** regularly

### CI/CD

1. **Use build cache** to speed up builds
2. **Build multi-arch** in parallel
3. **Sign images** for security
4. **Generate SBOM** for compliance

## References

- [Docker Buildx Documentation](https://docs.docker.com/buildx/)
- [Multi-Architecture Builds](https://docs.docker.com/buildx/multi-platform/)
- [OCI Image Specification](https://github.com/opencontainers/image-spec)
- [BlackLake Deployment Guide](DEPLOYMENT.md)