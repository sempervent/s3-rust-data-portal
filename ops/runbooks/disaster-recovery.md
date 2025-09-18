# BlackLake Disaster Recovery Runbook

## Overview

This runbook provides procedures for disaster recovery scenarios in the BlackLake system.

## Recovery Time Objectives (RTO)

- **Critical Systems**: 4 hours
- **Non-Critical Systems**: 24 hours
- **Data Recovery**: 1 hour

## Recovery Point Objectives (RPO)

- **Database**: 15 minutes
- **File Storage**: 1 hour
- **Configuration**: 24 hours

## Pre-Disaster Preparation

### 1. Backup Verification
```bash
# Verify backup integrity
just backup --verify

# Test restore process
just restore --dry-run latest

# Check backup retention
ls -la /var/backups/blacklake/
```

### 2. Documentation
- Keep runbooks updated
- Document all custom configurations
- Maintain contact lists
- Test communication channels

### 3. Monitoring
- Set up alerts for backup failures
- Monitor disk space
- Track backup completion
- Verify off-site storage

## Disaster Scenarios

### Complete System Failure

**Scenario**: Entire infrastructure is lost

**Recovery Steps:**

1. **Assess Damage**
   ```bash
   # Check what's available
   docker --version
   docker compose --version
   ls -la /var/backups/blacklake/
   ```

2. **Restore Infrastructure**
   ```bash
   # Clone repository
   git clone <repository-url>
   cd blacklake
   
   # Restore configuration
   cp .env.example .env
   # Update configuration as needed
   ```

3. **Restore Data**
   ```bash
   # Start minimal services
   docker compose up -d postgres minio
   
   # Wait for services to be ready
   sleep 60
   
   # Restore database
   just restore --db-only latest
   
   # Restore storage
   just restore --minio-only latest
   ```

4. **Restart Services**
   ```bash
   # Start all services
   docker compose up -d
   
   # Verify health
   just health
   ```

### Database Corruption

**Scenario**: Database is corrupted or inaccessible

**Recovery Steps:**

1. **Stop Services**
   ```bash
   docker compose stop api
   ```

2. **Restore Database**
   ```bash
   # Restore from backup
   just restore --db-only latest
   
   # Verify database integrity
   docker compose exec postgres psql -U postgres -d blacklake -c "SELECT count(*) FROM repositories;"
   ```

3. **Restart Services**
   ```bash
   docker compose start api
   ```

### Storage Failure

**Scenario**: MinIO/S3 storage is lost

**Recovery Steps:**

1. **Stop Services**
   ```bash
   docker compose stop api
   ```

2. **Restore Storage**
   ```bash
   # Restore from backup
   just restore --minio-only latest
   
   # Verify storage
   docker compose exec minio mc ls /data
   ```

3. **Restart Services**
   ```bash
   docker compose start api
   ```

### Partial Data Loss

**Scenario**: Some data is lost or corrupted

**Recovery Steps:**

1. **Identify Lost Data**
   ```bash
   # Check database integrity
   docker compose exec postgres psql -U postgres -d blacklake -c "SELECT * FROM repositories WHERE created_at > '2024-01-01';"
   
   # Check storage integrity
   docker compose exec minio mc ls /data
   ```

2. **Restore Specific Data**
   ```bash
   # Restore specific backup
   just restore --db-only 20240101_120000
   
   # Or restore specific files
   docker compose exec minio mc cp /backup/specific-file /data/
   ```

## Recovery Procedures

### 1. Infrastructure Recovery

```bash
# Set up new environment
git clone <repository-url>
cd blacklake

# Configure environment
cp .env.example .env
# Update configuration

# Start infrastructure
docker compose up -d postgres minio redis
```

### 2. Data Recovery

```bash
# Restore database
just restore --db-only latest

# Restore storage
just restore --minio-only latest

# Restore configuration
just restore --config-only latest
```

### 3. Service Recovery

```bash
# Start all services
docker compose up -d

# Verify health
just health

# Run tests
just test
```

### 4. Validation

```bash
# Check system health
curl -f http://localhost:8080/live
curl -f http://localhost:8080/ready

# Check data integrity
docker compose exec postgres psql -U postgres -d blacklake -c "SELECT count(*) FROM repositories;"
docker compose exec minio mc ls /data

# Run performance tests
just k6-load
```

## Communication During Disaster

### 1. Internal Communication
- Notify team immediately
- Create incident channel
- Update status page
- Document timeline

### 2. External Communication
- Notify users of outage
- Provide estimated recovery time
- Update status page regularly
- Communicate resolution

### 3. Stakeholder Updates
- Send regular updates
- Provide business impact assessment
- Share recovery progress
- Document lessons learned

## Testing Recovery Procedures

### 1. Regular Testing
- Monthly backup verification
- Quarterly restore testing
- Annual disaster recovery drill
- Continuous monitoring

### 2. Test Scenarios
- Complete system failure
- Database corruption
- Storage failure
- Network isolation

### 3. Documentation Updates
- Update runbooks based on tests
- Improve procedures
- Train team members
- Share lessons learned

## Recovery Checklist

### Pre-Recovery
- [ ] Assess damage
- [ ] Notify stakeholders
- [ ] Document timeline
- [ ] Gather resources

### During Recovery
- [ ] Stop affected services
- [ ] Restore infrastructure
- [ ] Restore data
- [ ] Restart services
- [ ] Verify functionality

### Post-Recovery
- [ ] Validate system health
- [ ] Run tests
- [ ] Monitor performance
- [ ] Update documentation
- [ ] Conduct review

## Emergency Contacts

- **On-call Engineer**: [Contact Info]
- **Engineering Manager**: [Contact Info]
- **Infrastructure Team**: [Contact Info]
- **Security Team**: [Contact Info]

## Useful Commands

```bash
# Quick recovery
just recover

# Verify backups
just backup --verify

# Test restore
just restore --dry-run latest

# Health check
just health

# Performance test
just k6-load
```
