# BlackLake Developer Commands

# ===== BUILD COMMANDS =====

# Build all Rust crates
build:
    cargo build

# build all bake images in parallel
bake:
    docker buildx bake local 

# Build in release mode
build-release:
    cargo build --release

# Build specific crate
build-crate crate:
    cargo build -p {{crate}}

# Run tests
test:
    cargo test

# Run tests with coverage
test-coverage:
    cargo tarpaulin --out Html

# Run clippy lints
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting
fmt-check:
    cargo fmt -- --check

# ===== DOCKER COMMANDS =====

# Build and push multi-arch images
bake-push:
    docker buildx bake --push all

# Build specific target
bake-target target:
    docker buildx bake --set {{target}}.output=type=docker {{target}}-local

# Build for specific platform
bake-platform platform target:
    docker buildx bake --set {{target}}.platform={{platform}} --set {{target}}.output=type=docker {{target}}-local

# ===== COMPOSE COMMANDS =====

# Start development stack
up-dev:
    docker compose --profile dev up -d --wait

# Start production stack
up-prod:
    docker compose --profile prod up -d --wait

# Start with specific profiles
up-profiles profiles:
    docker compose --profile {{profiles}} up -d --wait

# Stop all services
down:
    docker compose down

# Stop and remove volumes
down-clean:
    docker compose down -v

# View logs
logs service:
    docker compose logs -f {{service}}

# View all logs
logs-all:
    docker compose logs -f

# ===== DATABASE COMMANDS =====

# Run database migrations
migrate:
    docker compose exec api sqlx migrate run

# Create new migration
migrate-create name:
    docker compose exec api sqlx migrate add {{name}}

# Rollback last migration
migrate-rollback:
    docker compose exec api sqlx migrate revert

# Prepare SQLx offline data
sqlx-prepare:
    cargo sqlx prepare --workspace

# ===== TESTING COMMANDS =====

# Run end-to-end tests
e2e:
    docker compose --profile dev up -d --wait
    timeout 300 bash -c 'until curl -f http://localhost:8080/live; do sleep 5; done'
    timeout 300 bash -c 'until curl -f http://localhost:3000; do sleep 5; done'
    echo "Running E2E tests..."
    # Add your E2E test commands here
    docker compose down -v

# Run load tests
load-test:
    docker compose --profile dev up -d --wait
    timeout 300 bash -c 'until curl -f http://localhost:8080/live; do sleep 5; done'
    docker run --rm --network host -v $(pwd)/ops/k6:/scripts grafana/k6:latest run /scripts/load-test.js
    docker compose down -v

# Run security scans
scan:
    docker buildx bake --set api.output=type=docker --set ui.output=type=docker local
    docker run --rm -v /var/run/docker.sock:/var/run/docker.sock -v $(pwd):/workspace aquasec/trivy image blacklake-api:local
    docker run --rm -v /var/run/docker.sock:/var/run/docker.sock -v $(pwd):/workspace aquasec/trivy image blacklake-ui:local

# ===== DEVELOPMENT COMMANDS =====

# Start development environment
dev:
    docker compose --profile dev up -d --wait
    echo "Development environment ready!"
    echo "API: http://localhost:8080"
    echo "UI: http://localhost:3000"
    echo "MinIO: http://localhost:9001"
    echo "Keycloak: http://localhost:8081"

# Start with hot reload
dev-watch:
    docker compose --profile dev --profile ui-dev up -d --wait
    echo "Development environment with hot reload ready!"

# Clean development environment
dev-clean:
    docker compose down -v
    docker system prune -f
    cargo clean

# ===== BACKUP COMMANDS =====

# Create backup
db-backup:
    mkdir -p backups/$(date +%Y%m%d)
    docker compose exec -T db pg_dump -U blacklake blacklake > backups/$(date +%Y%m%d)/database.sql
    docker compose exec -T minio mc mirror minio/blacklake backups/$(date +%Y%m%d)/minio/
    tar -czf backups/blacklake-$(date +%Y%m%d).tar.gz backups/$(date +%Y%m%d)/

# Restore from backup
restore backup_file:
    tar -xzf {{backup_file}}
    docker compose exec -T db psql -U blacklake blacklake < backups/$(date +%Y%m%d)/database.sql
    docker compose exec -T minio mc mirror backups/$(date +%Y%m%d)/minio/ minio/blacklake/

# ===== MONITORING COMMANDS =====

# View system metrics
metrics:
    echo "Prometheus: http://localhost:9090"
    echo "Grafana: http://localhost:3001 (admin/admin)"
    echo "API Metrics: http://localhost:8080/metrics"

# View logs with structured output
logs-structured:
    docker compose logs -f --tail=100 | jq '.'

# ===== UTILITY COMMANDS =====

# Show service status
status:
    docker compose ps

# Show resource usage
stats:
    docker stats

# Show disk usage
disk-usage:
    docker system df

# Clean up unused resources
cleanup:
    docker system prune -f
    docker volume prune -f
    docker network prune -f

# ===== FAST SETUP COMMANDS =====

# Complete setup: build all images and start development stack
setup-all:
    @echo "🚀 Building all images and starting development stack..."
    docker buildx bake local
    docker compose --profile dev up -d --wait
    @echo "✅ Setup complete! API available at http://localhost:8080"
    @echo "📊 Grafana available at http://localhost:3000 (admin/admin)"
    @echo "🔍 Solr available at http://localhost:8983"

# Quick development setup (core services only)
setup-dev:
    @echo "⚡ Quick development setup..."
    docker buildx bake dev
    docker compose --profile dev up -d --wait
    @echo "✅ Development setup complete!"

# Production setup
setup-prod:
    @echo "🏭 Building production images..."
    docker buildx bake core
    docker compose --profile prod up -d --wait
    @echo "✅ Production setup complete!"

# Build all images with maximum parallelism
build-all:
    @echo "🔨 Building all images in parallel..."
    docker buildx bake --parallel all
    @echo "✅ All images built successfully!"

# Build core services only
build-core:
    @echo "🔨 Building core services..."
    docker buildx bake core
    @echo "✅ Core services built successfully!"

# ===== CI/CD COMMANDS =====

# Run full CI pipeline locally
ci:
    cargo fmt -- --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    docker buildx bake --set *.output=type=docker local
    docker compose --profile dev up -d --wait
    timeout 300 bash -c 'until curl -f http://localhost:8080/live; do sleep 5; done'
    echo "CI pipeline completed successfully!"

# Run security checks
security:
    cargo audit
    docker run --rm -v $(pwd):/workspace aquasec/trivy fs /workspace
    docker run --rm -v $(pwd):/workspace anchore/grype /workspace

# ===== DOCUMENTATION COMMANDS =====

# Generate API documentation
docs-api:
    cargo doc --open --no-deps

# Generate project documentation
docs:
    cargo doc --all --no-deps
    echo "Documentation generated in target/doc/"

# ===== RELEASE COMMANDS =====

# Create release
release version:
    git tag -a v{{version}} -m "Release v{{version}}"
    git push origin v{{version}}
    docker buildx bake --push --set *.tags=ghcr.io/blacklake/blacklake:v{{version}} all

# ===== HELP =====

# Show available commands
help:
    just --list

# Show command help
help-command command:
    just --show {{command}}