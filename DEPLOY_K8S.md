# Kubernetes Deployment Guide

## Overview

BlackLake can be deployed on Kubernetes using Helm charts with support for development, staging, and production environments.

## Prerequisites

- Kubernetes cluster (1.24+)
- Helm 3.0+
- kubectl configured
- Persistent storage class
- Ingress controller (optional)

## Quick Start

### Install with Helm

```bash
# Add BlackLake Helm repository
helm repo add blacklake https://charts.blacklake.dev
helm repo update

# Install BlackLake
helm install blacklake blacklake/blacklake \
  --namespace blacklake \
  --create-namespace \
  --values values.yaml
```

### Install with Kustomize

```bash
# Development environment
kubectl apply -k kustomize/overlays/dev

# Production environment
kubectl apply -k kustomize/overlays/prod
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `REDIS_URL` | Redis connection string | Required |
| `SOLR_URL` | Solr connection string | Required |
| `S3_ENDPOINT` | S3-compatible storage endpoint | Required |
| `S3_ACCESS_KEY_ID` | S3 access key | Required |
| `S3_SECRET_ACCESS_KEY` | S3 secret key | Required |
| `S3_BUCKET` | S3 bucket name | Required |
| `JWT_SECRET` | JWT signing secret | Required |

### Resource Requirements

#### Development
- CPU: 250m request, 500m limit
- Memory: 512Mi request, 1Gi limit

#### Production
- CPU: 1000m request, 2000m limit
- Memory: 2Gi request, 4Gi limit

## Helm Values

### Basic Configuration

```yaml
# values.yaml
image:
  repository: blacklake/blacklake
  tag: "latest"
  pullPolicy: IfNotPresent

service:
  type: ClusterIP
  port: 8080

ingress:
  enabled: true
  className: "nginx"
  hosts:
    - host: blacklake.local
      paths:
        - path: /
          pathType: Prefix
  tls: []

resources:
  limits:
    cpu: 1000m
    memory: 2Gi
  requests:
    cpu: 500m
    memory: 1Gi
```

### Autoscaling

```yaml
autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

hpa:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 4
        periodSeconds: 15
      selectPolicy: Max
```

### Pod Disruption Budget

```yaml
podDisruptionBudget:
  enabled: true
  minAvailable: 1
```

### Topology Spread Constraints

```yaml
topologySpreadConstraints:
  enabled: true
  maxSkew: 1
  topologyKey: kubernetes.io/hostname
  whenUnsatisfiable: DoNotSchedule
```

## Dependencies

### PostgreSQL

```yaml
postgresql:
  enabled: true
  auth:
    postgresPassword: "postgres"
    database: "blacklake"
  primary:
    persistence:
      enabled: true
      size: 20Gi
    resources:
      limits:
        cpu: 1000m
        memory: 2Gi
      requests:
        cpu: 500m
        memory: 1Gi
```

### Redis

```yaml
redis:
  enabled: true
  auth:
    enabled: false
  master:
    persistence:
      enabled: true
      size: 8Gi
    resources:
      limits:
        cpu: 500m
        memory: 1Gi
      requests:
        cpu: 250m
        memory: 512Mi
```

### Solr

```yaml
solr:
  enabled: true
  auth:
    enabled: false
  persistence:
    enabled: true
    size: 20Gi
  resources:
    limits:
      cpu: 1000m
      memory: 2Gi
    requests:
      cpu: 500m
      memory: 1Gi
```

## Monitoring

### ServiceMonitor

```yaml
serviceMonitor:
  enabled: true
  namespace: "monitoring"
  interval: 30s
  scrapeTimeout: 10s
  labels:
    app: blacklake
```

### Grafana Dashboard

```yaml
grafanaDashboard:
  enabled: true
  namespace: "monitoring"
  labels:
    app: blacklake
```

## Security

### Network Policies

```yaml
networkPolicy:
  enabled: true
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              name: ingress-nginx
      ports:
        - protocol: TCP
          port: 8080
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              name: postgresql
      ports:
        - protocol: TCP
          port: 5432
    - to:
        - namespaceSelector:
            matchLabels:
              name: redis
      ports:
        - protocol: TCP
          port: 6379
```

### Pod Security Context

```yaml
podSecurityContext:
  fsGroup: 1000
  runAsNonRoot: true
  runAsUser: 1000

securityContext:
  allowPrivilegeEscalation: false
  capabilities:
    drop:
    - ALL
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  runAsUser: 1000
```

## Troubleshooting

### Common Issues

1. **Pod CrashLoopBackOff**: Check logs and resource limits
2. **Database Connection**: Verify DATABASE_URL and network policies
3. **Storage Issues**: Check persistent volume claims
4. **Ingress Problems**: Verify ingress controller and TLS

### Debug Commands

```bash
# Check pod status
kubectl get pods -n blacklake

# View logs
kubectl logs -f deployment/blacklake -n blacklake

# Check events
kubectl get events -n blacklake --sort-by='.lastTimestamp'

# Port forward for local access
kubectl port-forward svc/blacklake 8080:8080 -n blacklake
```

### Health Checks

```bash
# Liveness probe
curl http://localhost:8080/live

# Readiness probe
curl http://localhost:8080/ready

# Health check
curl http://localhost:8080/health
```

## Production Checklist

- [ ] Resource limits configured
- [ ] Autoscaling enabled
- [ ] Pod disruption budget set
- [ ] Network policies applied
- [ ] Security context configured
- [ ] Monitoring enabled
- [ ] Backup strategy implemented
- [ ] TLS certificates configured
- [ ] Secrets management setup
- [ ] Log aggregation configured
