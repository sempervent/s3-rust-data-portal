# BlackLake Backup & Restore Runbook
# Week 5: Production backup and disaster recovery procedures

## Overview

This runbook provides step-by-step procedures for backing up and restoring BlackLake data using PgBackRest for PostgreSQL and MinIO lifecycle policies for object storage.

## Prerequisites

- PgBackRest installed and configured
- MinIO client (`mc`) installed
- Access to backup storage (S3 or local)
- Database superuser privileges
- MinIO admin credentials

## Backup Procedures

### 1. PostgreSQL Database Backup

#### Full Backup
```bash
# Create a full backup
sudo -u postgres pgbackrest --stanza=blacklake backup --type=full

# Verify backup
sudo -u postgres pgbackrest --stanza=blacklake info
```

#### Incremental Backup
```bash
# Create an incremental backup
sudo -u postgres pgbackrest --stanza=blacklake backup --type=incr

# Verify backup
sudo -u postgres pgbackrest --stanza=blacklake info
```

#### Differential Backup
```bash
# Create a differential backup
sudo -u postgres pgbackrest --stanza=blacklake backup --type=diff

# Verify backup
sudo -u postgres pgbackrest --stanza=blacklake info
```

### 2. MinIO Object Storage Backup

#### Backup to S3
```bash
# Configure MinIO client for S3
mc alias set s3 https://s3.amazonaws.com ACCESS_KEY SECRET_KEY

# Sync MinIO buckets to S3
mc mirror minio/blacklake s3/blacklake-backups/blacklake/
mc mirror minio/exports s3/blacklake-backups/exports/
mc mirror minio/mlflow s3/blacklake-backups/mlflow/

# Verify sync
mc ls s3/blacklake-backups/ --recursive
```

#### Local Backup
```bash
# Create local backup directory
mkdir -p /backup/minio/$(date +%Y%m%d)

# Copy MinIO data
cp -r /var/lib/minio/data/* /backup/minio/$(date +%Y%m%d)/

# Compress backup
tar -czf /backup/minio/blacklake-$(date +%Y%m%d).tar.gz /backup/minio/$(date +%Y%m%d)/
```

### 3. Configuration Backup

#### Backup Configuration Files
```bash
# Create config backup directory
mkdir -p /backup/config/$(date +%Y%m%d)

# Backup Docker Compose files
cp docker-compose.yml /backup/config/$(date +%Y%m%d)/
cp docker-compose.override.yml /backup/config/$(date +%Y%m%d)/
cp .env /backup/config/$(date +%Y%m%d)/

# Backup nginx configuration
cp -r ops/nginx/ /backup/config/$(date +%Y%m%d)/

# Backup PgBackRest configuration
cp ops/pgbackrest/pgbackrest.conf /backup/config/$(date +%Y%m%d)/

# Compress configuration backup
tar -czf /backup/config/blacklake-config-$(date +%Y%m%d).tar.gz /backup/config/$(date +%Y%m%d)/
```

## Restore Procedures

### 1. PostgreSQL Database Restore

#### Point-in-Time Recovery (PITR)
```bash
# Stop the database
sudo systemctl stop postgresql

# Restore to a specific point in time
sudo -u postgres pgbackrest --stanza=blacklake restore \
  --type=time \
  --target="2024-01-15 14:30:00" \
  --target-action=promote

# Start the database
sudo systemctl start postgresql

# Verify restore
sudo -u postgres psql -c "SELECT * FROM pg_stat_database WHERE datname='blacklake';"
```

#### Full Restore
```bash
# Stop the database
sudo systemctl stop postgresql

# Restore the latest full backup
sudo -u postgres pgbackrest --stanza=blacklake restore \
  --type=full \
  --target-action=promote

# Start the database
sudo systemctl start postgresql

# Verify restore
sudo -u postgres psql -c "SELECT * FROM pg_stat_database WHERE datname='blacklake';"
```

#### Restore to Different Location
```bash
# Create new data directory
sudo mkdir -p /var/lib/postgresql/restore

# Restore to different location
sudo -u postgres pgbackrest --stanza=blacklake restore \
  --type=full \
  --pg1-path=/var/lib/postgresql/restore \
  --target-action=promote

# Update PostgreSQL configuration to use new location
sudo -u postgres psql -c "ALTER SYSTEM SET data_directory = '/var/lib/postgresql/restore';"
```

### 2. MinIO Object Storage Restore

#### Restore from S3
```bash
# Configure MinIO client for S3
mc alias set s3 https://s3.amazonaws.com ACCESS_KEY SECRET_KEY

# Stop MinIO
docker compose stop minio

# Restore from S3
mc mirror s3/blacklake-backups/blacklake/ minio/blacklake/
mc mirror s3/blacklake-backups/exports/ minio/exports/
mc mirror s3/blacklake-backups/mlflow/ minio/mlflow/

# Start MinIO
docker compose start minio

# Verify restore
mc ls minio/blacklake/ --recursive
```

#### Restore from Local Backup
```bash
# Stop MinIO
docker compose stop minio

# Extract backup
tar -xzf /backup/minio/blacklake-20240115.tar.gz -C /

# Start MinIO
docker compose start minio

# Verify restore
mc ls minio/blacklake/ --recursive
```

### 3. Configuration Restore

#### Restore Configuration Files
```bash
# Stop services
docker compose down

# Extract configuration backup
tar -xzf /backup/config/blacklake-config-20240115.tar.gz -C /

# Restore files
cp /backup/config/20240115/docker-compose.yml ./
cp /backup/config/20240115/docker-compose.override.yml ./
cp /backup/config/20240115/.env ./
cp -r /backup/config/20240115/nginx/ ops/

# Start services
docker compose up -d
```

## Disaster Recovery Procedures

### 1. Complete System Recovery

#### Prerequisites
- New server with same specifications
- Network access to backup storage
- All required software installed

#### Recovery Steps
```bash
# 1. Install required software
sudo apt update
sudo apt install -y docker.io docker-compose postgresql-client

# 2. Clone repository
git clone https://github.com/blacklake/blacklake.git
cd blacklake

# 3. Restore configuration
tar -xzf /backup/config/blacklake-config-20240115.tar.gz -C ./

# 4. Start services
docker compose up -d

# 5. Restore database
sudo -u postgres pgbackrest --stanza=blacklake restore --type=full

# 6. Restore object storage
mc mirror s3/blacklake-backups/blacklake/ minio/blacklake/

# 7. Verify recovery
curl -f http://localhost:8080/live
curl -f http://localhost:3000
```

### 2. Partial Recovery

#### Database Only
```bash
# Stop database service
docker compose stop db

# Restore database
sudo -u postgres pgbackrest --stanza=blacklake restore --type=full

# Start database service
docker compose start db

# Verify database
sudo -u postgres psql -c "SELECT * FROM pg_stat_database WHERE datname='blacklake';"
```

#### Object Storage Only
```bash
# Stop MinIO service
docker compose stop minio

# Restore object storage
mc mirror s3/blacklake-backups/blacklake/ minio/blacklake/

# Start MinIO service
docker compose start minio

# Verify object storage
mc ls minio/blacklake/ --recursive
```

## Monitoring and Verification

### 1. Backup Verification

#### Check Backup Status
```bash
# Check PgBackRest status
sudo -u postgres pgbackrest --stanza=blacklake info

# Check MinIO backup status
mc ls s3/blacklake-backups/ --recursive

# Check backup logs
tail -f /var/log/pgbackrest/blacklake-backup.log
```

#### Verify Backup Integrity
```bash
# Verify PostgreSQL backup
sudo -u postgres pgbackrest --stanza=blacklake verify

# Verify MinIO backup
mc ls s3/blacklake-backups/blacklake/ --recursive | wc -l
mc ls minio/blacklake/ --recursive | wc -l
```

### 2. Restore Verification

#### Database Verification
```bash
# Check database connectivity
sudo -u postgres psql -c "SELECT version();"

# Check table counts
sudo -u postgres psql -d blacklake -c "SELECT schemaname, tablename, n_tup_ins, n_tup_upd, n_tup_del FROM pg_stat_user_tables;"

# Check recent activity
sudo -u postgres psql -d blacklake -c "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 10;"
```

#### Object Storage Verification
```bash
# Check MinIO connectivity
mc ls minio/

# Check bucket contents
mc ls minio/blacklake/ --recursive

# Check object integrity
mc stat minio/blacklake/some-object
```

## Troubleshooting

### Common Issues

#### PgBackRest Issues
```bash
# Check PgBackRest configuration
sudo -u postgres pgbackrest --stanza=blacklake info

# Check PgBackRest logs
tail -f /var/log/pgbackrest/blacklake-backup.log

# Restart PgBackRest service
sudo systemctl restart pgbackrest
```

#### MinIO Issues
```bash
# Check MinIO status
docker compose logs minio

# Check MinIO connectivity
mc admin info minio

# Restart MinIO service
docker compose restart minio
```

#### Network Issues
```bash
# Check network connectivity
ping s3.amazonaws.com

# Check DNS resolution
nslookup s3.amazonaws.com

# Check firewall rules
sudo ufw status
```

## Maintenance

### Regular Maintenance Tasks

#### Weekly Tasks
- Verify backup integrity
- Check backup retention policies
- Review backup logs
- Test restore procedures

#### Monthly Tasks
- Perform full disaster recovery drill
- Update backup documentation
- Review and update retention policies
- Test point-in-time recovery

#### Quarterly Tasks
- Review backup strategy
- Update backup procedures
- Test cross-region recovery
- Review and update disaster recovery plan

## Emergency Contacts

- **Database Administrator**: [Contact Info]
- **System Administrator**: [Contact Info]
- **Backup Administrator**: [Contact Info]
- **Emergency Hotline**: [Contact Info]

## References

- [PgBackRest Documentation](https://pgbackrest.org/)
- [MinIO Documentation](https://docs.min.io/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [BlackLake Operations Guide](../OPERATIONS.md)
