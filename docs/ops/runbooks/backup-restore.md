# Backup and Restore Runbook

## Overview
This runbook provides step-by-step procedures for backing up and restoring BlackLake data.

## Backup Procedures

### Database Backup
```bash
# Create database backup
pg_dump -h localhost -U blacklake -d blacklake > backup_$(date +%Y%m%d_%H%M%S).sql

# Compress backup
gzip backup_*.sql
```

### S3 Storage Backup
```bash
# Sync S3 bucket to backup location
aws s3 sync s3://blacklake-storage s3://blacklake-backup/$(date +%Y%m%d)/
```

### Redis Backup
```bash
# Create Redis backup
redis-cli --rdb /backup/redis_$(date +%Y%m%d_%H%M%S).rdb
```

## Restore Procedures

### Database Restore
```bash
# Restore from backup
gunzip -c backup_20240101_120000.sql.gz | psql -h localhost -U blacklake -d blacklake
```

### S3 Storage Restore
```bash
# Restore from backup
aws s3 sync s3://blacklake-backup/20240101/ s3://blacklake-storage/
```

### Redis Restore
```bash
# Stop Redis
systemctl stop redis

# Copy backup file
cp /backup/redis_20240101_120000.rdb /var/lib/redis/dump.rdb

# Start Redis
systemctl start redis
```

## Verification
- Verify database connectivity
- Check S3 bucket contents
- Test Redis operations
- Run health checks
