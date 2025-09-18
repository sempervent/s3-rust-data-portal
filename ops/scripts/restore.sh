#!/bin/bash
# BlackLake Restore Script
# Week 5: Disaster recovery and data restoration

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/blacklake}"
LOG_FILE="${BACKUP_DIR}/restore_$(date +%Y%m%d_%H%M%S).log"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
}

# Function to show usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS] BACKUP_DATE

Restore BlackLake from backup

OPTIONS:
    -h, --help          Show this help message
    -d, --dry-run       Show what would be restored without actually restoring
    -f, --force         Force restore even if services are running
    -c, --config-only   Restore only configuration files
    -b, --db-only       Restore only database
    -m, --minio-only    Restore only MinIO data
    -a, --app-only      Restore only application data

BACKUP_DATE:
    Date of backup to restore (format: YYYYMMDD_HHMMSS)
    Use 'latest' to restore the most recent backup

EXAMPLES:
    $0 latest                    # Restore latest backup
    $0 20240101_120000          # Restore specific backup
    $0 --dry-run latest         # Show what would be restored
    $0 --config-only latest     # Restore only configuration

EOF
}

# Function to find latest backup
find_latest_backup() {
    local latest_manifest=$(find "$BACKUP_DIR" -name "manifest_*.json" -type f | sort -r | head -n1)
    if [ -n "$latest_manifest" ]; then
        basename "$latest_manifest" | sed 's/manifest_\(.*\)\.json/\1/'
    else
        log_error "No backup manifests found in $BACKUP_DIR"
        exit 1
    fi
}

# Function to validate backup
validate_backup() {
    local backup_date=$1
    local manifest_file="${BACKUP_DIR}/manifest_${backup_date}.json"
    
    if [ ! -f "$manifest_file" ]; then
        log_error "Backup manifest not found: $manifest_file"
        exit 1
    fi
    
    log_info "Validating backup manifest: $manifest_file"
    
    # Check if all backup files exist
    local postgres_file="${BACKUP_DIR}/postgres_${backup_date}.sql.gz"
    local minio_file="${BACKUP_DIR}/minio_${backup_date}.tar.gz"
    local config_file="${BACKUP_DIR}/config_${backup_date}.tar.gz"
    local app_data_file="${BACKUP_DIR}/app_data_${backup_date}.tar.gz"
    
    local missing_files=()
    
    [ ! -f "$postgres_file" ] && missing_files+=("$postgres_file")
    [ ! -f "$minio_file" ] && missing_files+=("$minio_file")
    [ ! -f "$config_file" ] && missing_files+=("$config_file")
    [ ! -f "$app_data_file" ] && missing_files+=("$app_data_file")
    
    if [ ${#missing_files[@]} -gt 0 ]; then
        log_error "Missing backup files:"
        for file in "${missing_files[@]}"; do
            log_error "  - $file"
        done
        exit 1
    fi
    
    log_info "Backup validation successful"
}

# Function to check if services are running
check_services() {
    if [ "$FORCE" = false ]; then
        local running_services=()
        
        if docker compose ps postgres | grep -q "Up"; then
            running_services+=("postgres")
        fi
        
        if docker compose ps minio | grep -q "Up"; then
            running_services+=("minio")
        fi
        
        if docker compose ps api | grep -q "Up"; then
            running_services+=("api")
        fi
        
        if [ ${#running_services[@]} -gt 0 ]; then
            log_error "The following services are running and must be stopped before restore:"
            for service in "${running_services[@]}"; do
                log_error "  - $service"
            done
            log_error "Use --force to override this check"
            exit 1
        fi
    fi
}

# Function to restore database
restore_database() {
    local backup_date=$1
    local postgres_file="${BACKUP_DIR}/postgres_${backup_date}.sql.gz"
    
    log_info "Restoring PostgreSQL database..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would restore database from $postgres_file"
        return 0
    fi
    
    # Start PostgreSQL if not running
    if ! docker compose ps postgres | grep -q "Up"; then
        log_info "Starting PostgreSQL service..."
        docker compose up -d postgres
        sleep 10
    fi
    
    # Wait for PostgreSQL to be ready
    log_info "Waiting for PostgreSQL to be ready..."
    timeout 60 bash -c 'until docker compose exec postgres pg_isready -U postgres; do sleep 2; done'
    
    # Drop and recreate database
    log_info "Recreating database..."
    docker compose exec postgres psql -U postgres -c "DROP DATABASE IF EXISTS blacklake;"
    docker compose exec postgres psql -U postgres -c "CREATE DATABASE blacklake;"
    
    # Restore database
    log_info "Restoring database from backup..."
    gunzip -c "$postgres_file" | docker compose exec -T postgres psql -U postgres -d blacklake
    
    log_info "Database restore completed"
}

# Function to restore MinIO data
restore_minio() {
    local backup_date=$1
    local minio_file="${BACKUP_DIR}/minio_${backup_date}.tar.gz"
    
    log_info "Restoring MinIO data..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would restore MinIO data from $minio_file"
        return 0
    fi
    
    # Start MinIO if not running
    if ! docker compose ps minio | grep -q "Up"; then
        log_info "Starting MinIO service..."
        docker compose up -d minio
        sleep 10
    fi
    
    # Wait for MinIO to be ready
    log_info "Waiting for MinIO to be ready..."
    timeout 60 bash -c 'until curl -f http://localhost:9000/minio/health/live; do sleep 2; done'
    
    # Extract backup
    local temp_dir=$(mktemp -d)
    tar -xzf "$minio_file" -C "$temp_dir"
    
    # Restore data using MinIO client
    log_info "Restoring MinIO buckets..."
    docker compose exec -T minio mc mirror "$temp_dir" /data --overwrite
    
    # Cleanup
    rm -rf "$temp_dir"
    
    log_info "MinIO data restore completed"
}

# Function to restore configuration
restore_config() {
    local backup_date=$1
    local config_file="${BACKUP_DIR}/config_${backup_date}.tar.gz"
    
    log_info "Restoring configuration files..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would restore configuration from $config_file"
        return 0
    fi
    
    # Create backup of current configuration
    local current_config_backup="${BACKUP_DIR}/current_config_$(date +%Y%m%d_%H%M%S).tar.gz"
    log_info "Backing up current configuration to $current_config_backup"
    
    tar -czf "$current_config_backup" \
        -C "$PROJECT_ROOT" \
        ops/ .env docker-compose.yml docker-compose.override.yml \
        docker-bake.hcl justfile migrations/ 2>/dev/null || true
    
    # Extract configuration backup
    local temp_dir=$(mktemp -d)
    tar -xzf "$config_file" -C "$temp_dir"
    
    # Restore configuration files
    log_info "Restoring configuration files..."
    cp -r "$temp_dir/config_${backup_date}/ops" "$PROJECT_ROOT/"
    cp "$temp_dir/config_${backup_date}/.env" "$PROJECT_ROOT/" 2>/dev/null || true
    cp "$temp_dir/config_${backup_date}/docker-compose.yml" "$PROJECT_ROOT/"
    cp "$temp_dir/config_${backup_date}/docker-compose.override.yml" "$PROJECT_ROOT/" 2>/dev/null || true
    cp "$temp_dir/config_${backup_date}/docker-bake.hcl" "$PROJECT_ROOT/"
    cp "$temp_dir/config_${backup_date}/justfile" "$PROJECT_ROOT/"
    cp -r "$temp_dir/config_${backup_date}/migrations" "$PROJECT_ROOT/"
    
    # Cleanup
    rm -rf "$temp_dir"
    
    log_info "Configuration restore completed"
}

# Function to restore application data
restore_app_data() {
    local backup_date=$1
    local app_data_file="${BACKUP_DIR}/app_data_${backup_date}.tar.gz"
    
    log_info "Restoring application data..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would restore application data from $app_data_file"
        return 0
    fi
    
    # Extract backup
    local temp_dir=$(mktemp -d)
    tar -xzf "$app_data_file" -C "$temp_dir"
    
    # Restore application data
    if [ -d "$temp_dir/app_data_${backup_date}/data" ]; then
        log_info "Restoring application data directory..."
        cp -r "$temp_dir/app_data_${backup_date}/data" "$PROJECT_ROOT/"
    fi
    
    # Restore application logs
    if [ -f "$temp_dir/app_data_${backup_date}/application.log" ]; then
        log_info "Application logs available in backup (not restored automatically)"
    fi
    
    # Cleanup
    rm -rf "$temp_dir"
    
    log_info "Application data restore completed"
}

# Function to restart services
restart_services() {
    log_info "Restarting services..."
    
    if [ "$DRY_RUN" = true ]; then
        log_info "DRY RUN: Would restart services"
        return 0
    fi
    
    # Stop all services
    docker compose down
    
    # Start services
    docker compose up -d --wait
    
    # Wait for services to be healthy
    log_info "Waiting for services to be healthy..."
    timeout 120 bash -c 'until curl -f http://localhost:8080/live; do sleep 5; done'
    timeout 120 bash -c 'until curl -f http://localhost:8080/ready; do sleep 5; done'
    
    log_info "Services restarted successfully"
}

# Main restore function
main() {
    # Parse command line arguments
    local backup_date=""
    local DRY_RUN=false
    local FORCE=false
    local CONFIG_ONLY=false
    local DB_ONLY=false
    local MINIO_ONLY=false
    local APP_ONLY=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -d|--dry-run)
                DRY_RUN=true
                shift
                ;;
            -f|--force)
                FORCE=true
                shift
                ;;
            -c|--config-only)
                CONFIG_ONLY=true
                shift
                ;;
            -b|--db-only)
                DB_ONLY=true
                shift
                ;;
            -m|--minio-only)
                MINIO_ONLY=true
                shift
                ;;
            -a|--app-only)
                APP_ONLY=true
                shift
                ;;
            *)
                if [ -z "$backup_date" ]; then
                    backup_date="$1"
                else
                    log_error "Unknown argument: $1"
                    usage
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # Validate arguments
    if [ -z "$backup_date" ]; then
        log_error "Backup date is required"
        usage
        exit 1
    fi
    
    # Handle 'latest' backup
    if [ "$backup_date" = "latest" ]; then
        backup_date=$(find_latest_backup)
        log_info "Using latest backup: $backup_date"
    fi
    
    # Validate backup
    validate_backup "$backup_date"
    
    # Check services
    check_services
    
    log_info "Starting BlackLake restore process..."
    log_info "Backup date: $backup_date"
    log_info "Log file: $LOG_FILE"
    
    # Restore components based on options
    if [ "$CONFIG_ONLY" = true ]; then
        restore_config "$backup_date"
    elif [ "$DB_ONLY" = true ]; then
        restore_database "$backup_date"
    elif [ "$MINIO_ONLY" = true ]; then
        restore_minio "$backup_date"
    elif [ "$APP_ONLY" = true ]; then
        restore_app_data "$backup_date"
    else
        # Full restore
        restore_config "$backup_date"
        restore_database "$backup_date"
        restore_minio "$backup_date"
        restore_app_data "$backup_date"
        restart_services
    fi
    
    log_info "Restore process completed successfully"
}

# Run main function
main "$@"
