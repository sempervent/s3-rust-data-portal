# BlackLake Deployment Guide

This document provides comprehensive deployment instructions for the BlackLake data platform.

## Production Deployment

### Docker Compose

```bash
# Production deployment
docker-compose -f docker-compose.prod.yml up -d

# With custom configuration
BLACKLAKE_DOMAIN=blacklake.example.com docker-compose -f docker-compose.prod.yml up -d
```

### Kubernetes

```bash
# Deploy to Kubernetes
kubectl apply -f k8s/

# With Helm
helm install blacklake ./helm/blacklake -f helm/blacklake/values.yaml
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