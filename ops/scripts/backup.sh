#!/bin/bash
# BlackLake Backup Script
# Week 5: Comprehensive backup and disaster recovery

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKUP_DIR="${BACKUP_DIR:-/var/backups/blacklake}"
DATE=$(date +%Y%m%d_%H%M%S)
LOG_FILE="${BACKUP_DIR}/backup_${DATE}.log"

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

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Function to check if service is running
check_service() {
    local service=$1
    if docker compose ps "$service" | grep -q "Up"; then
        return 0
    else
        return 1
    fi
}

# Function to backup PostgreSQL database
backup_database() {
    log_info "Starting PostgreSQL backup..."
    
    if ! check_service "postgres"; then
        log_error "PostgreSQL service is not running"
        return 1
    fi
    
    # Create database backup
    local db_backup_file="${BACKUP_DIR}/postgres_${DATE}.sql"
    docker compose exec -T postgres pg_dump -U postgres -d blacklake > "$db_backup_file"
    
    # Compress backup
    gzip "$db_backup_file"
    
    log_info "PostgreSQL backup completed: ${db_backup_file}.gz"
    
    # Verify backup
    if [ -f "${db_backup_file}.gz" ]; then
        log_info "Backup verification: $(du -h "${db_backup_file}.gz" | cut -f1)"
    else
        log_error "Backup file not found: ${db_backup_file}.gz"
        return 1
    fi
}

# Function to backup MinIO data
backup_minio() {
    log_info "Starting MinIO backup..."
    
    if ! check_service "minio"; then
        log_error "MinIO service is not running"
        return 1
    fi
    
    # Create MinIO backup using mc (MinIO client)
    local minio_backup_dir="${BACKUP_DIR}/minio_${DATE}"
    mkdir -p "$minio_backup_dir"
    
    # Backup all buckets
    docker compose exec -T minio mc mirror /data "$minio_backup_dir" --overwrite
    
    # Compress backup
    tar -czf "${minio_backup_dir}.tar.gz" -C "$BACKUP_DIR" "minio_${DATE}"
    rm -rf "$minio_backup_dir"
    
    log_info "MinIO backup completed: ${minio_backup_dir}.tar.gz"
    
    # Verify backup
    if [ -f "${minio_backup_dir}.tar.gz" ]; then
        log_info "Backup verification: $(du -h "${minio_backup_dir}.tar.gz" | cut -f1)"
    else
        log_error "Backup file not found: ${minio_backup_dir}.tar.gz"
        return 1
    fi
}

# Function to backup configuration files
backup_config() {
    log_info "Starting configuration backup..."
    
    local config_backup_dir="${BACKUP_DIR}/config_${DATE}"
    mkdir -p "$config_backup_dir"
    
    # Backup important configuration files
    cp -r "$PROJECT_ROOT/ops" "$config_backup_dir/"
    cp "$PROJECT_ROOT/.env" "$config_backup_dir/" 2>/dev/null || true
    cp "$PROJECT_ROOT/docker-compose.yml" "$config_backup_dir/"
    cp "$PROJECT_ROOT/docker-compose.override.yml" "$config_backup_dir/" 2>/dev/null || true
    cp "$PROJECT_ROOT/docker-bake.hcl" "$config_backup_dir/"
    cp "$PROJECT_ROOT/justfile" "$config_backup_dir/"
    
    # Backup migrations
    cp -r "$PROJECT_ROOT/migrations" "$config_backup_dir/"
    
    # Compress backup
    tar -czf "${config_backup_dir}.tar.gz" -C "$BACKUP_DIR" "config_${DATE}"
    rm -rf "$config_backup_dir"
    
    log_info "Configuration backup completed: ${config_backup_dir}.tar.gz"
}

# Function to backup application data
backup_app_data() {
    log_info "Starting application data backup..."
    
    local app_backup_dir="${BACKUP_DIR}/app_data_${DATE}"
    mkdir -p "$app_backup_dir"
    
    # Backup application logs
    docker compose logs --no-color > "$app_backup_dir/application.log"
    
    # Backup application state (if any)
    if [ -d "$PROJECT_ROOT/data" ]; then
        cp -r "$PROJECT_ROOT/data" "$app_backup_dir/"
    fi
    
    # Compress backup
    tar -czf "${app_backup_dir}.tar.gz" -C "$BACKUP_DIR" "app_data_${DATE}"
    rm -rf "$app_backup_dir"
    
    log_info "Application data backup completed: ${app_backup_dir}.tar.gz"
}

# Function to create backup manifest
create_manifest() {
    log_info "Creating backup manifest..."
    
    local manifest_file="${BACKUP_DIR}/manifest_${DATE}.json"
    
    cat > "$manifest_file" << EOF
{
  "backup_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "backup_version": "1.0.0",
  "backup_type": "full",
  "components": {
    "postgres": {
      "backup_file": "postgres_${DATE}.sql.gz",
      "size": "$(du -b "${BACKUP_DIR}/postgres_${DATE}.sql.gz" 2>/dev/null | cut -f1 || echo "0")",
      "status": "$([ -f "${BACKUP_DIR}/postgres_${DATE}.sql.gz" ] && echo "success" || echo "failed")"
    },
    "minio": {
      "backup_file": "minio_${DATE}.tar.gz",
      "size": "$(du -b "${BACKUP_DIR}/minio_${DATE}.tar.gz" 2>/dev/null | cut -f1 || echo "0")",
      "status": "$([ -f "${BACKUP_DIR}/minio_${DATE}.tar.gz" ] && echo "success" || echo "failed")"
    },
    "config": {
      "backup_file": "config_${DATE}.tar.gz",
      "size": "$(du -b "${BACKUP_DIR}/config_${DATE}.tar.gz" 2>/dev/null | cut -f1 || echo "0")",
      "status": "$([ -f "${BACKUP_DIR}/config_${DATE}.tar.gz" ] && echo "success" || echo "failed")"
    },
    "app_data": {
      "backup_file": "app_data_${DATE}.tar.gz",
      "size": "$(du -b "${BACKUP_DIR}/app_data_${DATE}.tar.gz" 2>/dev/null | cut -f1 || echo "0")",
      "status": "$([ -f "${BACKUP_DIR}/app_data_${DATE}.tar.gz" ] && echo "success" || echo "failed")"
    }
  },
  "total_size": "$(du -b "${BACKUP_DIR}"/*_${DATE}.* 2>/dev/null | awk '{sum += $1} END {print sum}')",
  "backup_location": "$BACKUP_DIR"
}
EOF
    
    log_info "Backup manifest created: $manifest_file"
}

# Function to cleanup old backups
cleanup_old_backups() {
    log_info "Cleaning up old backups..."
    
    # Keep backups for 30 days
    find "$BACKUP_DIR" -name "*.gz" -mtime +30 -delete
    find "$BACKUP_DIR" -name "*.json" -mtime +30 -delete
    find "$BACKUP_DIR" -name "*.log" -mtime +30 -delete
    
    log_info "Old backups cleaned up"
}

# Function to send backup notification
send_notification() {
    local status=$1
    local message=$2
    
    # This would integrate with your notification system
    # For now, just log the notification
    log_info "Notification: $status - $message"
}

# Main backup function
main() {
    log_info "Starting BlackLake backup process..."
    log_info "Backup directory: $BACKUP_DIR"
    log_info "Log file: $LOG_FILE"
    
    local backup_success=true
    
    # Run backup components
    backup_database || backup_success=false
    backup_minio || backup_success=false
    backup_config || backup_success=false
    backup_app_data || backup_success=false
    
    # Create manifest
    create_manifest
    
    # Cleanup old backups
    cleanup_old_backups
    
    # Send notification
    if [ "$backup_success" = true ]; then
        log_info "Backup completed successfully"
        send_notification "SUCCESS" "BlackLake backup completed successfully"
    else
        log_error "Backup completed with errors"
        send_notification "ERROR" "BlackLake backup completed with errors"
        exit 1
    fi
    
    log_info "Backup process finished"
}

# Run main function
main "$@"
