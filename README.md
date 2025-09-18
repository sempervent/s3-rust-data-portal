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

## Architecture

Blacklake consists of six main crates:

- **`api`**: Axum HTTP server with REST endpoints
- **`core`**: Domain types, schemas, and business logic
- **`index`**: PostgreSQL database access layer
- **`storage`**: S3-compatible storage with presigned URLs
- **`modelx`**: ONNX/PyTorch metadata sniffers
- **`cli`**: Developer command-line interface

## Quick Start

### Prerequisites

- Rust 1.70+
- Docker and Docker Compose
- `just` command runner (optional, for dev commands)

### 1. Clone and Setup

```bash
git clone <repository-url>
cd blacklake
cp env.example .env
```

### 2. Start Infrastructure

```bash
# Start Postgres, MinIO, and Keycloak
docker compose up -d --wait

# Or using just
just up
```

### 3. Run Database Migrations

```bash
# Run migrations
just migrate

# Or manually
SQLX_OFFLINE=true sqlx migrate run
```

### 4. Build and Run

```bash
# Build the project
just build

# Run the API server
just run

# Or manually
cargo run -p api
```

The API server will be available at `http://localhost:8080`.

### 5. CLI-First Workflow

```bash
# Build CLI
cargo build -p blacklake-cli

# Create a repository
./target/debug/blacklake repo create my-data-repo

# Upload a file with interactive metadata
./target/debug/blacklake put my-data-repo main ./data/sample.csv --path datasets/sample.csv
# Follow the interactive prompts for metadata

# Edit metadata for an existing file
./target/debug/blacklake meta edit my-data-repo main datasets/sample.csv
# Opens editor or prompts for metadata updates

# Search your repository
./target/debug/blacklake search my-data-repo --org "MyLab" --file-type "text/csv" --sort size

# Download a file
./target/debug/blacklake get my-data-repo main datasets/sample.csv --out downloaded.csv

# View RDF metadata
./target/debug/blacklake rdf get my-data-repo main datasets/sample.csv --format turtle
```

## API Examples

### Create Repository

```bash
curl -X POST http://localhost:8080/v1/repos \
  -H "Content-Type: application/json" \
  -d '{"name": "my-models"}'
```

### Initialize Upload

```bash
curl -X POST http://localhost:8080/v1/repos/my-models/upload-init \
  -H "Content-Type: application/json" \
  -d '{
    "path": "models/resnet50.onnx",
    "size": 1024000,
    "media_type": "application/octet-stream"
  }'
```

### Upload File

```bash
# Use the presigned URL from upload-init response
curl -X PUT -T model.onnx "https://presigned-url-here"
```

### Create Commit

```bash
curl -X POST http://localhost:8080/v1/repos/my-models/commit \
  -H "Content-Type: application/json" \
  -d '{
    "ref": "main",
    "message": "Add ResNet-50 model",
    "changes": [
      {
        "op": "add",
        "path": "models/resnet50.onnx",
        "sha256": "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
        "meta": {
          "name": "ResNet-50 ONNX Model",
          "description": "Pre-trained ResNet-50 model",
          "version": "1.0.0",
          "tags": ["computer-vision", "image-classification"]
        }
      }
    ]
  }'
```

### Get Blob

```bash
curl http://localhost:8080/v1/repos/my-models/blob/main/models/resnet50.onnx
```

### List Tree

```bash
curl http://localhost:8080/v1/repos/my-models/tree/main
```

### Search

```bash
curl "http://localhost:8080/v1/repos/my-models/search?tags=computer-vision"
```

## CLI Usage

### Repository Management

```bash
# Create repository
blacklake repo create my-dataset

# List repositories
blacklake repo list
```

### Upload and Commit

```bash
# Initialize upload
blacklake upload-init \
  --repo my-dataset \
  --path data/train.json \
  --file ./train.json \
  --type application/json

# Upload file using the presigned URL
curl -X PUT -T train.json "https://presigned-url-here"

# Create commit
echo '[
  {
    "op": "add",
    "path": "data/train.json",
    "sha256": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
    "meta": {
      "name": "Training Data",
      "description": "Training dataset",
      "version": "1.0.0",
      "tags": ["dataset", "training"]
    }
  }
]' > changes.json

blacklake commit \
  --repo my-dataset \
  --ref main \
  --message "Add training data" \
  --changes changes.json
```

## Development

### Project Structure

```
blacklake/
├── crates/
│   ├── api/          # HTTP API server
│   ├── core/         # Domain types and schemas
│   ├── index/        # Database access layer
│   ├── storage/      # S3 storage adapter
│   ├── modelx/       # Model metadata extractors
│   └── cli/          # Command-line interface
├── migrations/       # Database migrations
├── docker-compose.yml
├── justfile         # Development commands
└── README.md
```

### Development Commands

```bash
# Build all crates
just build

# Run API server
just run

# Run tests
just test

# Check code
just check

# Run clippy
just clippy

# Format code
just fmt

# Start infrastructure
just up

# Stop infrastructure
just down

# Run migrations
just migrate

# Prepare sqlx data
just prepare
```

### Environment Variables

Copy `env.example` to `.env` and configure:

```bash
# Server
APP_HOST=0.0.0.0
APP_PORT=8080
RUST_LOG=info,blacklake=debug

# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres

# S3 / MinIO
S3_ENDPOINT=http://localhost:9000
S3_REGION=us-east-1
S3_BUCKET=blacklake
S3_ACCESS_KEY=minio
S3_SECRET_KEY=minio123
S3_FORCE_PATH_STYLE=true

# Auth (OIDC / JWT)
OIDC_ISSUER=http://localhost:8081/realms/master
OIDC_AUDIENCE=blacklake
```

## Database Schema

The system uses PostgreSQL with the following main tables:

- **`repo`**: Repository metadata
- **`ref`**: Branches and tags
- **`commit`**: Commit history
- **`object`**: Content-addressed objects
- **`entry`**: File/directory entries per commit
- **`acl`**: Access control lists
- **`audit_log`**: Audit trail

## Storage Layout

Objects are stored in S3 using content-addressed keys:

```
sha256/{first-2-chars}/{next-2-chars}/{full-sha256}
```

Example: `sha256/a6/65/a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3`

## Model Format Support

The `modelx` crate provides metadata extraction for:

- **ONNX**: Extracts opset version, producer info, IR version
- **PyTorch**: Detects version, model type, state dict presence

## Authentication

Currently uses mock authentication. In production, integrate with:

- OIDC providers (Keycloak, Auth0, etc.)
- JWT token validation
- Role-based access control

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run `just test` and `just clippy`
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Metadata & Dublin Core

Blacklake supports rich metadata management with three complementary approaches:

### 1. JSONB-in-DB (Source of Truth)
All metadata is stored as JSONB in the `entry.meta` column, providing flexibility and extensibility. This is the canonical source of truth for all artifact metadata.

### 2. Normalized Index Table (Fast Queries)
Common metadata fields are automatically indexed in the `entry_meta_index` table for fast filtering and sorting operations. This includes:
- `creation_dt`, `creator`, `file_name`, `file_type`, `file_size`
- `org_lab`, `description`, `data_source`, `data_collection_method`
- `version`, `notes`, `tags`, `license`

### 3. Optional RDF/Dublin Core
Artifacts can optionally generate RDF representations using Dublin Core metadata standards, stored in both JSON-LD and Turtle formats.

### Metadata Schema

The canonical metadata structure follows Dublin Core mapping:

**Required fields:**
- `creation_dt`: ISO 8601 timestamp
- `creator`: Creator email/identifier
- `file_name`: Human-readable file name
- `file_type`: MIME type
- `file_size`: Size in bytes
- `org_lab`: Organization/lab identifier
- `description`: Human-readable description
- `data_source`: Source of the data
- `data_collection_method`: How data was collected
- `version`: Version identifier

**Optional fields:**
- `notes`: Additional notes
- `tags`: Array of string tags
- `license`: License identifier

### Example Metadata

```json
{
  "creation_dt": "2025-01-17T18:28:00Z",
  "creator": "you@example.org",
  "file_name": "demo.csv",
  "file_type": "text/csv",
  "file_size": 1234,
  "org_lab": "ORNL",
  "description": "Demo dataset",
  "data_source": "sensor",
  "data_collection_method": "manual",
  "version": "1.0",
  "tags": ["demo", "csv"],
  "license": "CC-BY-4.0"
}
```

### RDF Generation

#### Commit with RDF Emission

```bash
curl -X POST \
  "http://localhost:8080/v1/repos/mylab/commit?emit_rdf=true" \
  -H 'Content-Type: application/json' \
  -d '{
    "ref": "main",
    "message": "add example with RDF",
    "changes": [{
      "op": "add",
      "path": "datasets/demo.csv",
      "sha256": "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
      "meta": {
        "creation_dt": "2025-01-17T18:28:00Z",
        "creator": "you@example.org",
        "file_name": "demo.csv",
        "file_type": "text/csv",
        "file_size": 1234,
        "org_lab": "ORNL",
        "description": "Demo dataset",
        "data_source": "sensor",
        "data_collection_method": "manual",
        "version": "1.0",
        "tags": ["demo", "csv"],
        "license": "CC-BY-4.0"
      }
    }]
  }'
```

#### Get RDF in Turtle Format

```bash
curl "http://localhost:8080/v1/repos/mylab/rdf/main/datasets/demo.csv?format=turtle"
```

#### Get RDF in JSON-LD Format

```bash
curl "http://localhost:8080/v1/repos/mylab/rdf/main/datasets/demo.csv?format=jsonld"
```

### CLI Usage

#### Commit with RDF Emission

```bash
blacklake commit \
  --repo mylab \
  --ref main \
  --message "Add dataset with RDF" \
  --put "datasets/demo.csv:sha256-hash:meta.json" \
  --emit-rdf
```

#### Get RDF

```bash
# Get Turtle format
blacklake rdf get mylab main datasets/demo.csv --format turtle > demo.ttl

# Get JSON-LD format
blacklake rdf get mylab main datasets/demo.csv --format jsonld > demo.jsonld
```

#### Set Repository Features

```bash
# Enable auto-RDF generation
blacklake repo features set mylab auto_rdf true
```

### Fast Search with Metadata Index

The metadata index enables fast filtering on common fields:

```bash
# Search by file type
curl "http://localhost:8080/v1/repos/mylab/search?file_type=text/csv"

# Search by organization
curl "http://localhost:8080/v1/repos/mylab/search?org_lab=ORNL"

# Search by tags
curl "http://localhost:8080/v1/repos/mylab/search?tags=demo"

# Search by date range
curl "http://localhost:8080/v1/repos/mylab/search?creation_dt_after=2025-01-01&creation_dt_before=2025-12-31"
```

### Dublin Core Mapping

The system maps canonical metadata to Dublin Core terms:

| Canonical Field | Dublin Core Term | Description |
|----------------|------------------|-------------|
| `file_name` | `dc:title` | Human-readable title |
| `creator` | `dc:creator` | Creator identifier |
| `description` | `dc:description` | Description |
| `creation_dt` | `dcterms:created` | Creation timestamp |
| `file_type` | `dc:format` | MIME type |
| `file_size` | `dcterms:extent` | Size in bytes |
| `data_source` | `dc:source` | Data source |
| `data_collection_method` | `dcterms:methodOfAccrual` | Collection method |
| `org_lab` | `dcterms:publisher` | Publisher/organization |
| `license` | `dcterms:license` | License |
| `tags[*]` | `dc:subject` | Subject tags |
| `version` | `dcterms:hasVersion` | Version |

## Security Model

### Authentication & Authorization

Blacklake implements a comprehensive security model with multiple layers:

#### OIDC/JWT Authentication
- **JWKS-based token verification** with automatic key rotation
- **Audience and issuer validation** for token integrity
- **Scope-based authorization** for fine-grained access control
- **Request ID tracking** for audit trails

#### Rate Limiting
- **Per-user rate limits** (configurable via `RATE_LIMIT_PER_USER`)
- **Per-IP rate limits** (configurable via `RATE_LIMIT_PER_IP`)
- **Global rate limits** with burst capacity
- **Automatic cleanup** of expired rate limit entries

#### Input Validation
- **Repository name validation** (alphanumeric, length limits, no path traversal)
- **Path normalization** and sanitization
- **Content-type validation** with dangerous type blocking
- **File size limits** with configurable maximums
- **Metadata schema validation** with Dublin Core compliance

#### Quota Management
- **Repository quotas**: size, file count, commit count limits
- **User quotas**: repository count, total storage limits
- **Real-time quota tracking** with usage history
- **Automatic quota enforcement** at commit and upload time

### Health & Monitoring

#### Health Endpoints
- **`/live`**: Liveness probe for container orchestration
- **`/ready`**: Readiness probe with dependency checks
- **`/metrics`**: Prometheus metrics for monitoring

#### Metrics
- **HTTP request metrics**: total requests, duration, status codes
- **Database metrics**: connection pool, query performance
- **S3 metrics**: operation counts, error rates
- **Custom metrics**: repository counts, storage usage

### Operations

#### Backup & Restore
```bash
# Database backup
pg_dump -h localhost -U blacklake blacklake > backup.sql

# S3 backup
aws s3 sync s3://blacklake s3://blacklake-backup --delete
```

#### Emergency Access
```bash
# Generate emergency token
curl -X POST http://localhost:8081/realms/master/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=blacklake" \
  -d "username=emergency-admin" \
  -d "password=[password]"
```

## Web UI

Blacklake includes a modern React-based web interface for managing data artifacts.

### Features
- **OIDC Authentication**: Secure login with OpenID Connect
- **Repository Management**: Create, browse, and manage repositories
- **File Operations**: Upload, download, and view files with metadata
- **RDF Preview**: View Dublin Core metadata in Turtle format
- **Search Interface**: Search across repositories with filters
- **Responsive Design**: Modern UI built with Tailwind CSS

### Quick Start
```bash
# Start the UI development server
cd ui
pnpm install
pnpm dev
```

The UI will be available at `http://localhost:5173`.

### Configuration
Copy `ui/env.example` to `ui/.env.development` and configure:
- `VITE_API_BASE_URL`: Blacklake API URL (default: http://localhost:8080)
- `VITE_OIDC_ISSUER`: OIDC provider URL (default: http://localhost:8081/realms/master)
- `VITE_OIDC_CLIENT_ID`: OIDC client ID (default: blacklake)

## Roadmap

- [x] Full OIDC/JWT authentication
- [x] Advanced search with faceted filters
- [x] Model format validation
- [x] Data lineage tracking
- [x] Backup and restore procedures
- [x] Metrics and monitoring
- [x] Web UI for repository browsing
- [ ] Multi-tenant support
- [ ] Advanced analytics dashboard