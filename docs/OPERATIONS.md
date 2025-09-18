# BlackLake Operations Guide
# Week 5: Production operations and maintenance

## Overview

This guide covers operational procedures for BlackLake in production environments, including monitoring, backups, disaster recovery, and maintenance.

## System Architecture

### Core Components

- **API Server**: Rust-based REST API
- **Database**: PostgreSQL 16 with PgBackRest
- **Object Storage**: MinIO (S3-compatible)
- **Authentication**: Keycloak (OIDC)
- **Monitoring**: Prometheus + Grafana
- **Search**: PostgreSQL trigram or OpenSearch
- **Background Jobs**: Redis-based queue system

### Network Architecture

```
Internet → Load Balancer → API Gateway → Services
                              ↓
                    ┌─────────────────┐
                    │   API Server    │
                    └─────────────────┘
                              ↓
                    ┌─────────────────┐
                    │   PostgreSQL    │
                    └─────────────────┘
                              ↓
                    ┌─────────────────┐
                    │     MinIO       │
                    └─────────────────┘
```

## Monitoring and Alerting

### Key Metrics

#### API Metrics
- Request rate (requests/second)
- Response time (P95, P99)
- Error rate (4xx, 5xx)
- Active connections
- Memory usage
- CPU usage

#### Database Metrics
- Connection count
- Query performance
- Cache hit ratio
- Disk usage
- Replication lag
- Lock waits

#### Storage Metrics
- Object count
- Storage usage
- Upload/download rates
- Success rates
- Bandwidth usage

#### Job Metrics
- Queue depth
- Processing time
- Success/failure rates
- Dead letter queue size

### Alerting Rules

#### Critical Alerts
- API error rate > 5%
- Database connection > 80%
- Disk usage > 90%
- Service down
- High memory usage

#### Warning Alerts
- API response time > 2s
- Queue depth > 100
- Cache hit ratio < 80%
- Disk usage > 80%

### Monitoring Setup

```bash
# Start monitoring stack
docker compose --profile prod up -d

# Access Grafana
open http://localhost:3001
# Login: admin/admin

# Access Prometheus
open http://localhost:9090
```

## Backup and Disaster Recovery

### Backup Strategy

#### Database Backups
- **Full backups**: Daily at 2 AM
- **Incremental backups**: Every 4 hours
- **WAL archiving**: Continuous
- **Retention**: 30 days full, 7 days incremental

#### Object Storage Backups
- **Replication**: Cross-region replication
- **Versioning**: Enabled for all buckets
- **Lifecycle**: 90 days standard, 1 year IA
- **Encryption**: AES-256

#### Configuration Backups
- **Daily**: Docker Compose files, environment configs
- **Version controlled**: All configs in Git
- **Encrypted**: Sensitive data encrypted at rest

### Backup Procedures

#### Automated Backups

```bash
# Daily backup script
#!/bin/bash
DATE=$(date +%Y%m%d)
BACKUP_DIR="/backups/$DATE"

# Create backup directory
mkdir -p $BACKUP_DIR

# Database backup
docker compose exec -T db pg_dump -U blacklake blacklake > $BACKUP_DIR/database.sql

# MinIO backup
docker compose exec -T minio mc mirror minio/blacklake $BACKUP_DIR/minio/

# Configuration backup
cp docker-compose.yml $BACKUP_DIR/
cp .env $BACKUP_DIR/

# Compress backup
tar -czf /backups/blacklake-$DATE.tar.gz $BACKUP_DIR/

# Upload to S3
aws s3 cp /backups/blacklake-$DATE.tar.gz s3://blacklake-backups/

# Cleanup local backup
rm -rf $BACKUP_DIR
```

#### Manual Backups

```bash
# Create backup
just backup

# Verify backup
tar -tzf backups/blacklake-20240115.tar.gz

# Test restore
just restore backups/blacklake-20240115.tar.gz
```

### Disaster Recovery

#### Recovery Time Objectives (RTO)
- **Critical systems**: 4 hours
- **Non-critical systems**: 24 hours
- **Full system**: 48 hours

#### Recovery Point Objectives (RPO)
- **Database**: 15 minutes
- **Object storage**: 1 hour
- **Configuration**: 24 hours

#### DR Procedures

1. **Assessment**
   - Identify affected systems
   - Assess data loss
   - Determine recovery strategy

2. **Recovery**
   - Restore from latest backup
   - Verify data integrity
   - Test system functionality

3. **Validation**
   - Run health checks
   - Verify user access
   - Monitor system performance

## Performance Optimization

### Database Optimization

#### Connection Pooling
```sql
-- PostgreSQL configuration
max_connections = 100
shared_buffers = 256MB
work_mem = 4MB
maintenance_work_mem = 64MB
effective_cache_size = 1GB
```

#### Indexing Strategy
```sql
-- Create indexes for common queries
CREATE INDEX CONCURRENTLY idx_entries_repo_path ON entries(repo_id, path);
CREATE INDEX CONCURRENTLY idx_entries_created_at ON entries(created_at);
CREATE INDEX CONCURRENTLY idx_audit_logs_user_action ON audit_logs(user_id, action);
```

#### Query Optimization
- Use EXPLAIN ANALYZE for slow queries
- Optimize JOIN operations
- Use appropriate data types
- Implement query result caching

### API Optimization

#### Caching Strategy
- **Redis**: Session data, frequently accessed data
- **Application**: In-memory caching for static data
- **CDN**: Static assets, API responses

#### Rate Limiting
```yaml
# Rate limiting configuration
rate_limits:
  api: 1000/hour
  upload: 100/hour
  download: 5000/hour
  search: 500/hour
```

### Storage Optimization

#### MinIO Configuration
```yaml
# MinIO optimization
minio:
  cache:
    drives: ["/tmp/cache"]
    expiry: 90
    max_use: 80
    quota: 0
```

#### Object Lifecycle
- **Standard**: 0-90 days
- **IA**: 90-365 days
- **Archive**: 365+ days
- **Delete**: After 7 years

## Security Operations

### Access Control

#### User Management
- **Admin users**: Full system access
- **Regular users**: Repository access
- **Service accounts**: API access only
- **Guest users**: Read-only access

#### Role-Based Access Control (RBAC)
```yaml
roles:
  admin:
    permissions: ["*"]
  user:
    permissions: ["read", "write", "commit"]
  viewer:
    permissions: ["read"]
```

### Security Monitoring

#### Audit Logging
- All user actions logged
- Failed authentication attempts
- Privilege escalation attempts
- Data access patterns

#### Security Scanning
```bash
# Daily security scans
just scan

# Vulnerability assessment
docker run --rm -v $(pwd):/workspace aquasec/trivy fs /workspace

# Dependency scanning
cargo audit
```

### Incident Response

#### Security Incident Procedures
1. **Detection**: Automated monitoring alerts
2. **Assessment**: Determine scope and impact
3. **Containment**: Isolate affected systems
4. **Eradication**: Remove threat
5. **Recovery**: Restore normal operations
6. **Lessons learned**: Document and improve

## Maintenance Procedures

### Regular Maintenance

#### Daily Tasks
- Monitor system health
- Check backup status
- Review error logs
- Verify service availability

#### Weekly Tasks
- Review performance metrics
- Update security patches
- Clean up old logs
- Test backup restore

#### Monthly Tasks
- Capacity planning
- Security assessment
- Performance optimization
- Disaster recovery testing

### System Updates

#### Rolling Updates
```bash
# Update API service
docker compose pull api
docker compose up -d api

# Update UI service
docker compose pull ui
docker compose up -d ui

# Update database (with downtime)
docker compose stop api
docker compose pull db
docker compose up -d db
docker compose start api
```

#### Zero-Downtime Updates
```bash
# Blue-green deployment
docker compose -f docker-compose.blue.yml up -d
# Test new deployment
# Switch traffic
docker compose -f docker-compose.green.yml down
```

### Capacity Planning

#### Resource Monitoring
- **CPU**: Monitor usage trends
- **Memory**: Track growth patterns
- **Storage**: Plan for data growth
- **Network**: Monitor bandwidth usage

#### Scaling Decisions
- **Horizontal**: Add more instances
- **Vertical**: Increase resource limits
- **Database**: Read replicas, sharding
- **Storage**: Distributed storage

## Troubleshooting

### Common Issues

#### High CPU Usage
```bash
# Check process usage
docker stats

# Analyze API performance
curl http://localhost:8080/metrics | grep cpu

# Check database queries
docker compose exec db psql -U blacklake -c "SELECT * FROM pg_stat_activity;"
```

#### Memory Issues
```bash
# Check memory usage
docker stats

# Analyze memory leaks
docker compose exec api valgrind --tool=memcheck

# Check database memory
docker compose exec db psql -U blacklake -c "SELECT * FROM pg_stat_bgwriter;"
```

#### Storage Issues
```bash
# Check disk usage
df -h

# Check MinIO status
docker compose exec minio mc admin info minio

# Clean up old data
docker compose exec minio mc rm --recursive --force minio/blacklake/old/
```

#### Network Issues
```bash
# Check connectivity
ping google.com

# Check DNS resolution
nslookup api.blacklake.com

# Check firewall rules
iptables -L
```

### Performance Issues

#### Slow API Response
1. Check database performance
2. Analyze query execution plans
3. Review caching effectiveness
4. Check network latency

#### High Database Load
1. Identify slow queries
2. Check index usage
3. Review connection pooling
4. Consider read replicas

#### Storage Performance
1. Check disk I/O
2. Review MinIO configuration
3. Consider SSD storage
4. Optimize object lifecycle

## Emergency Procedures

### Service Outage

#### Immediate Response
1. **Assess**: Determine scope and impact
2. **Communicate**: Notify stakeholders
3. **Contain**: Prevent further issues
4. **Restore**: Bring services back online

#### Recovery Steps
```bash
# Check service status
just status

# Restart failed services
docker compose restart api

# Check logs
just logs api

# Verify functionality
curl http://localhost:8080/live
```

### Data Corruption

#### Detection
- Checksum verification
- Data integrity checks
- User reports
- Monitoring alerts

#### Recovery
1. **Stop**: Halt affected services
2. **Assess**: Determine corruption scope
3. **Restore**: Use latest clean backup
4. **Verify**: Test data integrity
5. **Resume**: Restart services

### Security Breach

#### Immediate Response
1. **Isolate**: Disconnect affected systems
2. **Assess**: Determine breach scope
3. **Contain**: Prevent further access
4. **Investigate**: Analyze attack vector

#### Recovery
1. **Patch**: Fix security vulnerabilities
2. **Reset**: Change all passwords
3. **Restore**: Use clean backups
4. **Monitor**: Enhanced security monitoring

## Documentation and Runbooks

### Operational Runbooks
- [Backup and Restore](ops/runbooks/backup-restore.md)
- [Disaster Recovery](ops/runbooks/disaster-recovery.md)
- [Security Incident Response](ops/runbooks/security-incident.md)
- [Performance Troubleshooting](ops/runbooks/performance-troubleshooting.md)

### Monitoring Dashboards
- [System Overview](grafana/dashboards/system-overview.json)
- [API Performance](grafana/dashboards/api-performance.json)
- [Database Metrics](grafana/dashboards/database-metrics.json)
- [Storage Metrics](grafana/dashboards/storage-metrics.json)

### Alerting Rules
- [Critical Alerts](prometheus/alerts/critical.yml)
- [Warning Alerts](prometheus/alerts/warning.yml)
- [Performance Alerts](prometheus/alerts/performance.yml)

## Support and Escalation

### Support Levels
- **L1**: Basic troubleshooting
- **L2**: Advanced technical support
- **L3**: Engineering team
- **L4**: Vendor support

### Escalation Procedures
1. **L1**: Check documentation, basic troubleshooting
2. **L2**: Advanced diagnostics, configuration changes
3. **L3**: Code changes, architecture decisions
4. **L4**: Vendor engagement, critical issues

### Contact Information
- **On-call**: +1-555-BLACKLAKE
- **Email**: support@blacklake.com
- **Slack**: #blacklake-support
- **Emergency**: +1-555-BLACKLAKE-911