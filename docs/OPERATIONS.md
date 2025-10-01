# BlackLake Operations Guide

This document provides comprehensive operations procedures for the BlackLake data platform.

## Operations Procedures

### Health Monitoring

```bash
# Check all services
curl http://localhost:8080/health

# Check database
curl http://localhost:8080/health/db

# Check storage
curl http://localhost:8080/health/storage
```

### Backup and Recovery

```bash
# Backup database
docker-compose exec db pg_dump -U blacklake blacklake > backup.sql

# Restore database
docker-compose exec -T db psql -U blacklake blacklake < backup.sql
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