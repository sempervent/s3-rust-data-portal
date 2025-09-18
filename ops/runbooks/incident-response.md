# BlackLake Incident Response Runbook

## Overview

This runbook provides step-by-step procedures for responding to incidents in the BlackLake system.

## Incident Severity Levels

### P1 - Critical
- System completely down
- Data loss or corruption
- Security breach
- **Response Time**: 15 minutes
- **Resolution Time**: 4 hours

### P2 - High
- Major functionality unavailable
- Performance degradation > 50%
- **Response Time**: 1 hour
- **Resolution Time**: 24 hours

### P3 - Medium
- Minor functionality issues
- Performance degradation < 50%
- **Response Time**: 4 hours
- **Resolution Time**: 72 hours

### P4 - Low
- Cosmetic issues
- Feature requests
- **Response Time**: 24 hours
- **Resolution Time**: 1 week

## Initial Response

### 1. Acknowledge the Incident
```bash
# Check system status
curl -f http://localhost:8080/live
curl -f http://localhost:8080/ready

# Check service health
docker compose ps
docker compose logs --tail=100
```

### 2. Assess Impact
- How many users are affected?
- What functionality is impacted?
- Is data at risk?
- What is the business impact?

### 3. Communicate
- Update status page
- Notify stakeholders
- Create incident channel
- Document timeline

## Common Incidents

### API Service Down

**Symptoms:**
- HTTP 5xx errors
- Connection timeouts
- Health checks failing

**Diagnosis:**
```bash
# Check API container
docker compose ps api
docker compose logs api --tail=100

# Check resource usage
docker stats api

# Check database connectivity
docker compose exec api sqlx migrate info
```

**Resolution:**
```bash
# Restart API service
docker compose restart api

# If restart fails, rebuild
docker compose up -d --build api

# Check logs
docker compose logs api -f
```

### Database Issues

**Symptoms:**
- Connection errors
- Slow queries
- Lock timeouts

**Diagnosis:**
```bash
# Check database status
docker compose exec postgres pg_isready

# Check connections
docker compose exec postgres psql -U postgres -c "SELECT count(*) FROM pg_stat_activity;"

# Check locks
docker compose exec postgres psql -U postgres -c "SELECT * FROM pg_locks WHERE NOT granted;"
```

**Resolution:**
```bash
# Restart database
docker compose restart postgres

# Check for long-running queries
docker compose exec postgres psql -U postgres -c "SELECT pid, now() - pg_stat_activity.query_start AS duration, query FROM pg_stat_activity WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes';"

# Kill problematic queries if necessary
docker compose exec postgres psql -U postgres -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND state = 'active' AND query_start < now() - interval '10 minutes';"
```

### Storage Issues

**Symptoms:**
- Upload failures
- File not found errors
- S3/MinIO errors

**Diagnosis:**
```bash
# Check MinIO status
docker compose ps minio
docker compose logs minio --tail=100

# Check disk space
df -h
docker system df

# Test MinIO connectivity
docker compose exec minio mc ls /data
```

**Resolution:**
```bash
# Restart MinIO
docker compose restart minio

# Check bucket status
docker compose exec minio mc ls /data

# Clean up old data if needed
docker compose exec minio mc rm --recursive --force /data/old-bucket
```

### Performance Issues

**Symptoms:**
- Slow response times
- High CPU/memory usage
- Timeout errors

**Diagnosis:**
```bash
# Check resource usage
docker stats

# Check application metrics
curl http://localhost:9090/metrics | grep blacklake

# Check database performance
docker compose exec postgres psql -U postgres -c "SELECT * FROM pg_stat_statements ORDER BY total_time DESC LIMIT 10;"
```

**Resolution:**
```bash
# Scale services if needed
docker compose up -d --scale api=3

# Check for memory leaks
docker compose logs api | grep -i "out of memory"

# Restart services
docker compose restart
```

## Escalation Procedures

### When to Escalate
- P1 incidents not resolved in 2 hours
- P2 incidents not resolved in 8 hours
- Data loss or corruption
- Security incidents
- External dependencies down

### Escalation Contacts
- **On-call Engineer**: [Contact Info]
- **Engineering Manager**: [Contact Info]
- **Product Manager**: [Contact Info]
- **Security Team**: [Contact Info]

## Post-Incident

### 1. Incident Review
- Root cause analysis
- Timeline reconstruction
- Impact assessment
- Lessons learned

### 2. Documentation
- Update runbooks
- Create knowledge base articles
- Update monitoring alerts
- Improve procedures

### 3. Follow-up Actions
- Implement fixes
- Update monitoring
- Train team members
- Schedule reviews

## Emergency Contacts

- **PagerDuty**: [Contact Info]
- **Slack**: #blacklake-incidents
- **Status Page**: [URL]
- **Monitoring**: [URL]

## Useful Commands

```bash
# Quick health check
just health

# View all logs
just logs

# Restart all services
just restart

# Backup before changes
just backup

# Restore from backup
just restore latest

# Performance testing
just k6-load

# Security scan
just scan
```
