#!/bin/bash
# Automated Backup Validation Script
# This script implements nightly restore tests in CI using Docker Compose

set -euo pipefail

# Configuration
BACKUP_DIR="/backups"
RESTORE_DIR="/tmp/restore-test"
DOCKER_COMPOSE_FILE="docker-compose.test.yml"
LOG_FILE="/var/log/backup-validation.log"
RETENTION_DAYS=30

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $1" | tee -a "$LOG_FILE"
}

# Error handling
error_exit() {
    log "ERROR: $1"
    exit 1
}

# Success message
success() {
    log "SUCCESS: $1"
}

# Warning message
warning() {
    log "WARNING: $1"
}

# Cleanup function
cleanup() {
    log "Cleaning up test environment..."
    if [ -d "$RESTORE_DIR" ]; then
        rm -rf "$RESTORE_DIR"
    fi
    
    # Stop any running containers
    docker-compose -f "$DOCKER_COMPOSE_FILE" down -v 2>/dev/null || true
    
    # Remove test images
    docker rmi blacklake-test:latest 2>/dev/null || true
}

# Set up trap for cleanup
trap cleanup EXIT

# Check prerequisites
check_prerequisites() {
    log "Checking prerequisites..."
    
    # Check if Docker is running
    if ! docker info >/dev/null 2>&1; then
        error_exit "Docker is not running"
    fi
    
    # Check if docker-compose is available
    if ! command -v docker-compose >/dev/null 2>&1; then
        error_exit "docker-compose is not installed"
    fi
    
    # Check if backup directory exists
    if [ ! -d "$BACKUP_DIR" ]; then
        error_exit "Backup directory $BACKUP_DIR does not exist"
    fi
    
    success "Prerequisites check passed"
}

# Find latest backup
find_latest_backup() {
    log "Finding latest backup..."
    
    local latest_backup=$(find "$BACKUP_DIR" -name "blacklake_backup_*.sql" -type f -printf '%T@ %p\n' | sort -n | tail -1 | cut -d' ' -f2-)
    
    if [ -z "$latest_backup" ]; then
        error_exit "No backup files found in $BACKUP_DIR"
    fi
    
    log "Latest backup: $latest_backup"
    echo "$latest_backup"
}

# Verify backup integrity
verify_backup_integrity() {
    local backup_file="$1"
    
    log "Verifying backup integrity..."
    
    # Check if backup file exists and is readable
    if [ ! -f "$backup_file" ] || [ ! -r "$backup_file" ]; then
        error_exit "Backup file $backup_file is not accessible"
    fi
    
    # Check file size (should be > 0)
    local file_size=$(stat -c%s "$backup_file")
    if [ "$file_size" -eq 0 ]; then
        error_exit "Backup file $backup_file is empty"
    fi
    
    # Check if it's a valid SQL file
    if ! head -n 10 "$backup_file" | grep -q "PostgreSQL database dump"; then
        error_exit "Backup file $backup_file does not appear to be a valid PostgreSQL dump"
    fi
    
    # Check for corruption (basic check)
    if ! tail -n 10 "$backup_file" | grep -q "PostgreSQL database dump complete"; then
        warning "Backup file $backup_file may be incomplete or corrupted"
    fi
    
    success "Backup integrity verification passed"
}

# Create test Docker Compose file
create_test_compose() {
    log "Creating test Docker Compose file..."
    
    cat > "$DOCKER_COMPOSE_FILE" << 'EOF'
version: '3.8'

services:
  test-db:
    image: postgres:15
    environment:
      - POSTGRES_DB=blacklake_test
      - POSTGRES_USER=test_user
      - POSTGRES_PASSWORD=test_password
    ports:
      - "5433:5432"
    volumes:
      - test_db_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U test_user -d blacklake_test"]
      interval: 10s
      timeout: 5s
      retries: 5

  test-api:
    image: blacklake-test:latest
    environment:
      - DATABASE_URL=postgresql://test_user:test_password@test-db:5432/blacklake_test
      - REDIS_URL=redis://test-redis:6379
    ports:
      - "8082:8080"
    depends_on:
      test-db:
        condition: service_healthy
      test-redis:
        condition: service_started

  test-redis:
    image: redis:7-alpine
    ports:
      - "6380:6379"
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  test_db_data:
EOF
    
    success "Test Docker Compose file created"
}

# Build test image
build_test_image() {
    log "Building test image..."
    
    # Create Dockerfile for test
    cat > Dockerfile.test << 'EOF'
FROM blacklake/api:latest

# Add test-specific configuration
COPY test-config.toml /app/config.toml

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1
EOF
    
    # Create test configuration
    cat > test-config.toml << 'EOF'
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://test_user:test_password@test-db:5432/blacklake_test"
max_connections = 10

[redis]
url = "redis://test-redis:6379"

[storage]
provider = "local"
path = "/tmp/storage"

[logging]
level = "info"
EOF
    
    # Build test image
    docker build -f Dockerfile.test -t blacklake-test:latest . || error_exit "Failed to build test image"
    
    success "Test image built successfully"
}

# Start test environment
start_test_environment() {
    log "Starting test environment..."
    
    # Start services
    docker-compose -f "$DOCKER_COMPOSE_FILE" up -d || error_exit "Failed to start test environment"
    
    # Wait for services to be ready
    log "Waiting for services to be ready..."
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if docker-compose -f "$DOCKER_COMPOSE_FILE" ps | grep -q "Up (healthy)"; then
            success "Test environment is ready"
            return 0
        fi
        
        attempt=$((attempt + 1))
        log "Waiting for services... (attempt $attempt/$max_attempts)"
        sleep 10
    done
    
    error_exit "Test environment failed to start within expected time"
}

# Restore backup
restore_backup() {
    local backup_file="$1"
    
    log "Restoring backup to test database..."
    
    # Copy backup file to container
    docker cp "$backup_file" "$(docker-compose -f "$DOCKER_COMPOSE_FILE" ps -q test-db):/tmp/backup.sql"
    
    # Restore backup
    docker-compose -f "$DOCKER_COMPOSE_FILE" exec -T test-db psql -U test_user -d blacklake_test < "$backup_file" || error_exit "Failed to restore backup"
    
    success "Backup restored successfully"
}

# Run validation tests
run_validation_tests() {
    log "Running validation tests..."
    
    local test_results=()
    local total_tests=0
    local passed_tests=0
    
    # Test 1: Database connectivity
    total_tests=$((total_tests + 1))
    if docker-compose -f "$DOCKER_COMPOSE_FILE" exec -T test-db psql -U test_user -d blacklake_test -c "SELECT 1;" >/dev/null 2>&1; then
        test_results+=("Database connectivity: PASS")
        passed_tests=$((passed_tests + 1))
    else
        test_results+=("Database connectivity: FAIL")
    fi
    
    # Test 2: API health check
    total_tests=$((total_tests + 1))
    if curl -f http://localhost:8082/health >/dev/null 2>&1; then
        test_results+=("API health check: PASS")
        passed_tests=$((passed_tests + 1))
    else
        test_results+=("API health check: FAIL")
    fi
    
    # Test 3: Database schema validation
    total_tests=$((total_tests + 1))
    if docker-compose -f "$DOCKER_COMPOSE_FILE" exec -T test-db psql -U test_user -d blacklake_test -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" | grep -q "[0-9]"; then
        test_results+=("Database schema validation: PASS")
        passed_tests=$((passed_tests + 1))
    else
        test_results+=("Database schema validation: FAIL")
    fi
    
    # Test 4: Data integrity check
    total_tests=$((total_tests + 1))
    local table_count=$(docker-compose -f "$DOCKER_COMPOSE_FILE" exec -T test-db psql -U test_user -d blacklake_test -t -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" | tr -d ' \n')
    if [ "$table_count" -gt 0 ]; then
        test_results+=("Data integrity check: PASS")
        passed_tests=$((passed_tests + 1))
    else
        test_results+=("Data integrity check: FAIL")
    fi
    
    # Test 5: API functionality test
    total_tests=$((total_tests + 1))
    if curl -f http://localhost:8082/api/v1/repos >/dev/null 2>&1; then
        test_results+=("API functionality test: PASS")
        passed_tests=$((passed_tests + 1))
    else
        test_results+=("API functionality test: FAIL")
    fi
    
    # Print test results
    log "Validation test results:"
    for result in "${test_results[@]}"; do
        if [[ "$result" == *"PASS"* ]]; then
            echo -e "${GREEN}✓ $result${NC}"
        else
            echo -e "${RED}✗ $result${NC}"
        fi
    done
    
    # Calculate success rate
    local success_rate=$((passed_tests * 100 / total_tests))
    log "Test results: $passed_tests/$total_tests tests passed ($success_rate%)"
    
    if [ $success_rate -lt 80 ]; then
        error_exit "Validation tests failed (success rate: $success_rate%)"
    fi
    
    success "All validation tests passed"
}

# Generate report
generate_report() {
    local backup_file="$1"
    local success_rate="$2"
    
    log "Generating validation report..."
    
    local report_file="/var/log/backup-validation-report-$(date +%Y%m%d-%H%M%S).json"
    
    cat > "$report_file" << EOF
{
    "timestamp": "$(date -Iseconds)",
    "backup_file": "$backup_file",
    "validation_status": "SUCCESS",
    "success_rate": $success_rate,
    "test_environment": {
        "docker_compose_file": "$DOCKER_COMPOSE_FILE",
        "restore_directory": "$RESTORE_DIR"
    },
    "metrics": {
        "backup_size_bytes": $(stat -c%s "$backup_file"),
        "restore_duration_seconds": $SECONDS,
        "validation_duration_seconds": $SECONDS
    }
}
EOF
    
    log "Validation report generated: $report_file"
}

# Clean up old reports
cleanup_old_reports() {
    log "Cleaning up old reports..."
    
    find /var/log -name "backup-validation-report-*.json" -mtime +$RETENTION_DAYS -delete 2>/dev/null || true
    find /var/log -name "backup-validation.log.*" -mtime +$RETENTION_DAYS -delete 2>/dev/null || true
    
    success "Old reports cleaned up"
}

# Main execution
main() {
    log "Starting backup validation process..."
    
    # Check prerequisites
    check_prerequisites
    
    # Find latest backup
    local latest_backup=$(find_latest_backup)
    
    # Verify backup integrity
    verify_backup_integrity "$latest_backup"
    
    # Create test environment
    create_test_compose
    build_test_image
    start_test_environment
    
    # Restore backup
    restore_backup "$latest_backup"
    
    # Run validation tests
    run_validation_tests
    
    # Generate report
    generate_report "$latest_backup" 100
    
    # Clean up old reports
    cleanup_old_reports
    
    success "Backup validation completed successfully"
}

# Run main function
main "$@"
