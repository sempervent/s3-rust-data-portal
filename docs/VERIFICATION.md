# BlackLake Verification Guide

This document provides comprehensive verification procedures for all BlackLake systems and components.

## System Verification Checklist

### ✅ **Infrastructure Components**

#### **Database (PostgreSQL)**
- [ ] Database connection established
- [ ] Migration scripts executed successfully
- [ ] Connection pooling configured
- [ ] Health checks responding
- [ ] Query performance optimized
- [ ] Backup procedures tested

#### **Storage (S3/MinIO)**
- [ ] S3 connection established
- [ ] Bucket creation and configuration
- [ ] Lifecycle policies applied
- [ ] Encryption enabled
- [ ] Presigned URL generation
- [ ] Health checks responding

#### **Search (Solr)**
- [ ] Solr connection established
- [ ] Document indexing working
- [ ] Search queries responding
- [ ] Suggestions working
- [ ] Reindexing functional
- [ ] Health checks responding

#### **Cache (Redis)**
- [ ] Redis connection established
- [ ] Cache operations working
- [ ] TTL configuration correct
- [ ] Statistics collection working
- [ ] Health checks responding

### ✅ **API Endpoints**

#### **Authentication**
- [ ] JWT token validation
- [ ] OIDC integration
- [ ] Rate limiting working
- [ ] Request timeouts configured
- [ ] Circuit breaker functional

#### **Repository Operations**
- [ ] Create repository
- [ ] List repositories
- [ ] Get repository details
- [ ] Update repository
- [ ] Delete repository

#### **File Operations**
- [ ] Upload files
- [ ] Download files
- [ ] List files
- [ ] Delete files
- [ ] File metadata extraction

#### **Search Operations**
- [ ] Text search
- [ ] Metadata search
- [ ] Filtered search
- [ ] Pagination
- [ ] Search suggestions

## Verification Procedures

### **1. System Health Checks**

```bash
# Check all services
curl http://localhost:8080/health

# Check database
curl http://localhost:8080/health/db

# Check storage
curl http://localhost:8080/health/storage

# Check search
curl http://localhost:8080/health/search

# Check cache
curl http://localhost:8080/health/cache
```

### **2. Authentication Verification**

```bash
# Get JWT token
TOKEN=$(curl -s -X POST http://localhost:8081/realms/blacklake/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=admin&password=admin&grant_type=password&client_id=blacklake" | jq -r .access_token)

# Test authenticated request
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/api/v1/repos
```

## Additional Resources

- [Local Testing Guide](local_testing.md)
- [Fast Setup Guide](FAST_SETUP.md)
- [Migration Setup](MIGRATION_SETUP.md)
- [Project Status](PROJECT_STATUS.md)
- [Implementation Summary](IMPLEMENTATION_SUMMARY.md)

## Support

- **Documentation**: [Documentation Home](index.md)
- **Issues**: [GitHub Issues](https://github.com/your-org/blacklake/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/blacklake/discussions)