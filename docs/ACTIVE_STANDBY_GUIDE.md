# Active-Standby Guide for BlackLake

## Overview

This guide provides comprehensive instructions for setting up and managing active-standby configurations for BlackLake API with database replication, including failover procedures and promotion protocols.

## Architecture

### Active-Standby Setup

```
┌─────────────────┐    ┌─────────────────┐
│   Active API    │    │  Standby API    │
│   (Primary)     │    │  (Secondary)    │
└─────────────────┘    └─────────────────┘
         │                       │
         │                       │
         ▼                       ▼
┌─────────────────┐    ┌─────────────────┐
│  Primary DB     │◄───┤  Standby DB     │
│  (Read/Write)   │    │  (Read Only)    │
└─────────────────┘    └─────────────────┘
```

### Components

1. **Active API Server**: Handles all read/write operations
2. **Standby API Server**: Ready to take over in case of failure
3. **Primary Database**: Master database with read/write access
4. **Standby Database**: Replica database with read-only access
5. **Load Balancer**: Routes traffic to active server
6. **Health Monitoring**: Monitors system health and triggers failover

## Setup Instructions

### 1. Database Replication Setup

#### PostgreSQL Streaming Replication

```bash
# On Primary Database
# Configure postgresql.conf
wal_level = replica
max_wal_senders = 3
max_replication_slots = 3
wal_keep_segments = 64

# Configure pg_hba.conf
host replication replicator 10.0.0.0/8 md5

# Create replication user
sudo -u postgres psql
CREATE USER replicator WITH REPLICATION ENCRYPTED PASSWORD 'replication_password';
```

#### Standby Database Configuration

```bash
# On Standby Database
# Create base backup
pg_basebackup -h primary_host -D /var/lib/postgresql/data -U replicator -v -P -W

# Configure recovery.conf
standby_mode = 'on'
primary_conninfo = 'host=primary_host port=5432 user=replicator password=replication_password'
trigger_file = '/tmp/postgresql.trigger'
```

### 2. API Server Configuration

#### Active Server Configuration

```yaml
# docker-compose.active.yml
version: '3.8'
services:
  api:
    image: blacklake/api:latest
    environment:
      - DATABASE_URL=postgresql://user:pass@primary_db:5432/blacklake
      - REDIS_URL=redis://redis:6379
      - ROLE=active
    ports:
      - "8080:8080"
    depends_on:
      - primary_db
      - redis

  primary_db:
    image: postgres:15
    environment:
      - POSTGRES_DB=blacklake
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_data:/var/lib/postgresql/data
```

#### Standby Server Configuration

```yaml
# docker-compose.standby.yml
version: '3.8'
services:
  api:
    image: blacklake/api:latest
    environment:
      - DATABASE_URL=postgresql://user:pass@standby_db:5432/blacklake
      - REDIS_URL=redis://redis:6379
      - ROLE=standby
    ports:
      - "8081:8080"
    depends_on:
      - standby_db
      - redis

  standby_db:
    image: postgres:15
    environment:
      - POSTGRES_DB=blacklake
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=pass
    volumes:
      - postgres_standby_data:/var/lib/postgresql/data
```

### 3. Load Balancer Configuration

#### Nginx Configuration

```nginx
# /etc/nginx/sites-available/blacklake
upstream blacklake_backend {
    server active_api:8080 max_fails=3 fail_timeout=30s;
    server standby_api:8080 backup;
}

server {
    listen 80;
    server_name blacklake.example.com;

    location / {
        proxy_pass http://blacklake_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Health check configuration
        proxy_next_upstream error timeout invalid_header http_500 http_502 http_503 http_504;
        proxy_next_upstream_tries 2;
        proxy_next_upstream_timeout 10s;
    }

    location /health {
        proxy_pass http://blacklake_backend/health;
        access_log off;
    }
}
```

## Failover Procedures

### 1. Automatic Failover

#### Health Check Configuration

```yaml
# health-check.yml
version: '3.8'
services:
  health-monitor:
    image: blacklake/health-monitor:latest
    environment:
      - ACTIVE_API_URL=http://active_api:8080
      - STANDBY_API_URL=http://standby_api:8080
      - FAILOVER_THRESHOLD=3
      - CHECK_INTERVAL=10s
    volumes:
      - ./failover-script.sh:/scripts/failover.sh
```

#### Failover Script

```bash
#!/bin/bash
# failover-script.sh

ACTIVE_API="http://active_api:8080"
STANDBY_API="http://standby_api:8080"
FAILOVER_THRESHOLD=3
CHECK_INTERVAL=10

check_health() {
    local api_url=$1
    curl -f -s "$api_url/health" > /dev/null
    return $?
}

failover() {
    echo "Initiating failover to standby server..."
    
    # Update load balancer configuration
    sed -i 's/server active_api:8080/server standby_api:8080/' /etc/nginx/sites-available/blacklake
    nginx -s reload
    
    # Promote standby database
    ssh standby_db "touch /tmp/postgresql.trigger"
    
    # Update API configurations
    docker-compose -f docker-compose.standby.yml up -d
    
    echo "Failover completed"
}

# Main monitoring loop
failure_count=0
while true; do
    if ! check_health $ACTIVE_API; then
        failure_count=$((failure_count + 1))
        echo "Health check failed. Count: $failure_count"
        
        if [ $failure_count -ge $FAILOVER_THRESHOLD ]; then
            failover
            break
        fi
    else
        failure_count=0
    fi
    
    sleep $CHECK_INTERVAL
done
```

### 2. Manual Failover

#### Step-by-Step Manual Failover

1. **Verify Standby Status**
   ```bash
   # Check standby database replication lag
   psql -h standby_db -c "SELECT pg_last_wal_receive_lsn(), pg_last_wal_replay_lsn();"
   ```

2. **Stop Active API**
   ```bash
   docker-compose -f docker-compose.active.yml down
   ```

3. **Promote Standby Database**
   ```bash
   # On standby database server
   touch /tmp/postgresql.trigger
   ```

4. **Update Load Balancer**
   ```bash
   # Update nginx configuration
   sed -i 's/server active_api:8080/server standby_api:8080/' /etc/nginx/sites-available/blacklake
   nginx -s reload
   ```

5. **Start Standby API**
   ```bash
   docker-compose -f docker-compose.standby.yml up -d
   ```

6. **Verify Failover**
   ```bash
   curl -f http://blacklake.example.com/health
   ```

## Promotion Procedures

### 1. Standby to Active Promotion

#### Pre-Promotion Checklist

- [ ] Verify standby database is up-to-date
- [ ] Check replication lag is minimal (< 1 second)
- [ ] Ensure standby API is healthy
- [ ] Verify all services are running
- [ ] Test connectivity to all dependencies

#### Promotion Steps

1. **Stop Active Services**
   ```bash
   docker-compose -f docker-compose.active.yml down
   ```

2. **Promote Database**
   ```bash
   # On standby database
   touch /tmp/postgresql.trigger
   ```

3. **Update Configuration**
   ```bash
   # Update environment variables
   export ROLE=active
   export DATABASE_URL=postgresql://user:pass@standby_db:5432/blacklake
   ```

4. **Start Active Services**
   ```bash
   docker-compose -f docker-compose.active.yml up -d
   ```

5. **Update Load Balancer**
   ```bash
   # Point load balancer to new active server
   sed -i 's/server standby_api:8080/server active_api:8080/' /etc/nginx/sites-available/blacklake
   nginx -s reload
   ```

### 2. Post-Promotion Validation

#### Health Checks

```bash
# Check API health
curl -f http://active_api:8080/health

# Check database connectivity
curl -f http://active_api:8080/health/database

# Check storage connectivity
curl -f http://active_api:8080/health/storage

# Check Redis connectivity
curl -f http://active_api:8080/health/redis
```

#### Functional Tests

```bash
# Test basic API operations
curl -X GET http://active_api:8080/api/v1/repos
curl -X POST http://active_api:8080/api/v1/repos -d '{"name":"test"}'
curl -X GET http://active_api:8080/api/v1/repos/test
```

## Monitoring and Alerting

### 1. Health Monitoring

#### Custom Health Endpoints

```rust
// crates/api/src/health.rs
pub async fn cluster_health(
    State(state): State<HealthState>,
) -> Result<Json<ClusterHealth>, ApiError> {
    let mut health = ClusterHealth {
        status: "healthy".to_string(),
        primary: ServiceStatus {
            status: "active".to_string(),
            last_check: chrono::Utc::now(),
            replication_lag: 0.0,
        },
        standby: ServiceStatus {
            status: "standby".to_string(),
            last_check: chrono::Utc::now(),
            replication_lag: 0.0,
        },
    };

    // Check primary database
    match check_primary_database(&state).await {
        Ok(lag) => {
            health.primary.replication_lag = lag;
        }
        Err(_) => {
            health.primary.status = "unhealthy".to_string();
            health.status = "degraded".to_string();
        }
    }

    // Check standby database
    match check_standby_database(&state).await {
        Ok(lag) => {
            health.standby.replication_lag = lag;
        }
        Err(_) => {
            health.standby.status = "unhealthy".to_string();
        }
    }

    Ok(Json(health))
}
```

### 2. Replication Lag Monitoring

```sql
-- Check replication lag
SELECT 
    client_addr,
    state,
    pg_wal_lsn_diff(pg_current_wal_lsn(), sent_lsn) AS sent_lag,
    pg_wal_lsn_diff(pg_current_wal_lsn(), flush_lsn) AS flush_lag,
    pg_wal_lsn_diff(pg_current_wal_lsn(), replay_lsn) AS replay_lag
FROM pg_stat_replication;
```

### 3. Alerting Configuration

```yaml
# alertmanager.yml
route:
  group_by: ['alertname']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
- name: 'web.hook'
  webhook_configs:
  - url: 'http://alertmanager:9093/api/v1/alerts'
    send_resolved: true

# Prometheus rules
groups:
- name: blacklake.rules
  rules:
  - alert: HighReplicationLag
    expr: pg_replication_lag > 10
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High replication lag detected"
      description: "Replication lag is {{ $value }} seconds"

  - alert: DatabaseDown
    expr: up{job="postgres"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Database is down"
      description: "Database {{ $labels.instance }} is not responding"
```

## Recovery Procedures

### 1. Full System Recovery

1. **Restore from Backup**
   ```bash
   # Restore database
   pg_restore -h localhost -U user -d blacklake /backup/blacklake_backup.sql
   
   # Restore storage
   aws s3 sync s3://blacklake-backup/ s3://blacklake-storage/
   ```

2. **Reconfigure Services**
   ```bash
   # Update configuration files
   # Start services
   docker-compose up -d
   ```

3. **Validate Recovery**
   ```bash
   # Run health checks
   # Test functionality
   # Verify data integrity
   ```

### 2. Partial Recovery

1. **Identify Affected Components**
2. **Restore Specific Services**
3. **Validate Partial Recovery**
4. **Monitor System Health**

## Best Practices

### 1. Regular Testing

- **Monthly Failover Tests**: Test failover procedures monthly
- **Quarterly DR Drills**: Full disaster recovery testing
- **Annual Recovery Testing**: Complete system recovery testing

### 2. Monitoring

- **Continuous Health Monitoring**: 24/7 health checks
- **Replication Lag Alerts**: Immediate alerts for lag > 5 seconds
- **Performance Monitoring**: Track system performance metrics

### 3. Documentation

- **Keep Documentation Updated**: Regular updates to procedures
- **Version Control**: Track changes to configurations
- **Runbook Maintenance**: Keep runbooks current and tested

## Troubleshooting

### Common Issues

1. **Replication Lag**
   - Check network connectivity
   - Verify database configuration
   - Monitor system resources

2. **Failover Failures**
   - Check service dependencies
   - Verify configuration files
   - Test connectivity

3. **Data Inconsistency**
   - Compare checksums
   - Check replication status
   - Verify backup integrity

### Support Contacts

- **Primary DBA**: dba@company.com
- **System Administrator**: sysadmin@company.com
- **On-Call Engineer**: +1-555-ONCALL
