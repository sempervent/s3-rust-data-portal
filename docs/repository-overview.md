# Blacklake

A Rust-based, S3-backed, Git-style data artifact service for managing machine learning models and datasets with version control, content addressing, and metadata search capabilities.

## Features

- **Git-style Version Control**: Commit, branch, and tag your data artifacts
- **Content-Addressed Storage**: SHA256-based deduplication with S3 backend
- **Metadata Search**: JSON Schema validation and PostgreSQL JSONB search
- **Model Format Support**: ONNX and PyTorch metadata extraction
- **RESTful API**: HTTP API with JWT/OIDC authentication
- **Developer CLI**: Command-line interface for common operations
- **Docker Compose**: Complete development environment with Postgres, MinIO, and Keycloak
- **Multi-Arch Images**: AMD64 and ARM64 support with Docker Buildx
- **Production Ready**: Comprehensive monitoring, security, and operations tooling
- **Performance Testing**: K6-based load, stress, and spike testing

## Architecture

Blacklake consists of six main crates:

- **`api`**: Axum HTTP server with REST endpoints
- **`core`**: Domain types, schemas, and business logic
- **`index`**: PostgreSQL database access layer
- **`storage`**: S3-compatible storage with presigned URLs
- **`modelx`**: ONNX/PyTorch metadata sniffers
- **`cli`**: Developer command-line interface

## 📚 Documentation

**Comprehensive documentation is available in this site:**

- **[Project Status](PROJECT_STATUS.md)** - Current project status and completion summary
- **[Fast Setup](FAST_SETUP.md)** - Quick setup guide with build optimization
- **[Verification](VERIFICATION.md)** - Comprehensive verification of all systems
- **[Implementation Summary](IMPLEMENTATION_SUMMARY.md)** - Week-by-week implementation details
- **[Deployment](DEPLOYMENT.md)** - Production deployment guide
- **[Operations](OPERATIONS.md)** - Operations runbooks and procedures

## Quick Start

### Prerequisites

- Rust 1.70+
- Docker and Docker Compose
- `just` command runner (optional, for dev commands)

## Local Initialization

The `blacklake init` command helps you initialize directories and files as BlackLake artifacts with comprehensive metadata templates.

### Basic Initialization

```bash
# Initialize a new BlackLake repository
blacklake init

# Initialize with a specific name
blacklake init --name "my-ml-project"

# Initialize with custom metadata
blacklake init --name "my-project" --description "Machine Learning Pipeline"
```

### Advanced Initialization

```bash
# Initialize with comprehensive metadata
blacklake init \
  --name "advanced-ml-project" \
  --description "Advanced Machine Learning Pipeline with MLOps" \
  --author "Data Science Team" \
  --license "MIT" \
  --tags "ml,ai,pipeline,production" \
  --version "1.0.0"
```

### Initialization Options

The `blacklake init` command supports various options:

```bash
blacklake init [OPTIONS]

Options:
  -n, --name <NAME>              Repository name
  -d, --description <DESCRIPTION> Repository description
  -a, --author <AUTHOR>          Repository author
  -l, --license <LICENSE>        Repository license
  -t, --tags <TAGS>              Comma-separated tags
  -v, --version <VERSION>        Repository version
  -f, --force                    Overwrite existing repository
  -h, --help                     Print help
```

### Generated Structure

After initialization, you'll have:

```
my-project/
├── .blacklake/
│   ├── config.toml          # Repository configuration
│   ├── metadata.json       # Repository metadata
│   └── .gitignore          # BlackLake-specific gitignore
├── data/                    # Your data files
├── models/                  # ML models
├── datasets/               # Training datasets
└── README.md               # Project documentation
```

### Configuration File

The `.blacklake/config.toml` file contains:

```toml
[repository]
name = "my-project"
description = "Machine Learning Pipeline"
author = "Data Science Team"
license = "MIT"
version = "1.0.0"
tags = ["ml", "ai", "pipeline"]

[storage]
backend = "s3"
bucket = "blacklake"
region = "us-east-1"

[search]
backend = "postgres"
index_metadata = true
index_content = true

[auth]
provider = "oidc"
issuer = "https://keycloak.example.com/realms/blacklake"
```

### Metadata Template

The `metadata.json` file provides a comprehensive template:

```json
{
  "repository": {
    "name": "my-project",
    "description": "Machine Learning Pipeline",
    "author": "Data Science Team",
    "license": "MIT",
    "version": "1.0.0",
    "tags": ["ml", "ai", "pipeline"],
    "created_at": "2024-01-15T10:00:00Z",
    "updated_at": "2024-01-15T10:00:00Z"
  },
  "data": {
    "format": "mixed",
    "size": "0B",
    "files": 0,
    "directories": 0
  },
  "models": {
    "count": 0,
    "formats": [],
    "total_size": "0B"
  },
  "datasets": {
    "count": 0,
    "formats": [],
    "total_size": "0B"
  },
  "dependencies": {
    "python": [],
    "r": [],
    "julia": [],
    "other": []
  },
  "environment": {
    "os": "linux",
    "python_version": "3.11",
    "r_version": "4.3",
    "julia_version": "1.9"
  },
  "workflow": {
    "stages": [],
    "pipeline": [],
    "artifacts": []
  },
  "compliance": {
    "data_classification": "internal",
    "retention_policy": "7y",
    "access_control": "team",
    "audit_logging": true
  }
}
```

## Development Setup

### 1. Clone and Build

```bash
git clone https://github.com/NAERM/s3-rust-data-portal.git
cd s3-rust-data-portal

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace
```

### 2. Start Services

```bash
# Start all services
docker-compose up -d

# Or use just (if installed)
just up-dev
```

### 3. Initialize Repository

```bash
# Initialize a new repository
blacklake init --name "my-ml-project"

# Add some data
echo "Hello, BlackLake!" > data/hello.txt
blacklake add data/hello.txt

# Commit the changes
blacklake commit -m "Initial commit"
```

## API Usage

### Authentication

```bash
# Get JWT token from Keycloak
curl -X POST http://localhost:8081/realms/blacklake/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=admin&password=admin&grant_type=password&client_id=blacklake"
```

### Repository Operations

```bash
# Create repository
curl -X POST http://localhost:8080/api/v1/repos \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "my-repo", "description": "My ML Repository"}'

# List repositories
curl -X GET http://localhost:8080/api/v1/repos \
  -H "Authorization: Bearer $JWT_TOKEN"

# Upload file
curl -X POST http://localhost:8080/api/v1/repos/my-repo/upload \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -F "file=@data/model.onnx"
```

### Search Operations

```bash
# Search files
curl -X GET "http://localhost:8080/api/v1/search?q=model&type=onnx" \
  -H "Authorization: Bearer $JWT_TOKEN"

# Search by metadata
curl -X POST http://localhost:8080/api/v1/search \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": {"metadata.tags": "production"}, "limit": 10}'
```

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

## Monitoring

### Health Checks

```bash
# API health
curl http://localhost:8080/health

# Database health
curl http://localhost:8080/health/db

# Storage health
curl http://localhost:8080/health/storage
```

### Metrics

- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)
- **Jaeger**: http://localhost:16686

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see [LICENSE](https://github.com/NAERM/s3-rust-data-portal/blob/main/LICENSE) file for details.

## Support

- **Documentation**: [Home](index.md)
- **Issues**: https://github.com/NAERM/s3-rust-data-portal/issues
- **Discussions**: https://github.com/NAERM/s3-rust-data-portal/discussions


