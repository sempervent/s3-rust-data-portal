# BlackLake Migration and CLI Setup

This document explains how to handle database migrations and use the BlackLake CLI in the Docker Compose setup.

## 🗄️ Database Migrations

### How Migrations Work

The migration system is designed to run **before** any application services start, ensuring the database schema is always up-to-date.

### Migration Files

All migration files are located in the `migrations/` directory:

- `0001_init.sql` - Initial database schema
- `0002_metadata_index_and_rdf.sql` - Metadata and RDF support
- `0003_governance_features.sql` - Governance features
- `0003_lineage_and_quotas.sql` - Lineage and quota management
- `0004_multitenant_abac.sql` - Multi-tenant access control
- `0005_data_classification.sql` - Data classification
- `0006_external_sources.sql` - External data sources
- `0007_compliance_features.sql` - Compliance features
- `0008_compliance_jobs.sql` - Compliance job processing
- `0009_fix_missing_columns.sql` - Schema fixes
- `0010_api_missing_tables.sql` - API-specific tables

### Running Migrations

#### Option 1: Automatic (Recommended)
Migrations run automatically when you start the services:

```bash
# Start all services (migrations run first)
docker-compose up -d

# Start with specific profiles
docker-compose --profile dev up -d
```

#### Option 2: Manual Migration
Run migrations manually:

```bash
# Run migrations only
docker-compose run --rm migrations

# Or use the migration script directly
docker-compose run --rm migrations /app/scripts/run-migrations.sh
```

#### Option 3: One-time Migration
For production deployments:

```bash
# Run migrations and exit
docker-compose run --rm migrate
```

### Migration Dependencies

The migration system ensures proper ordering:

1. **Database** starts first
2. **Migrations** run after database is healthy
3. **API and other services** start after migrations complete

## 🖥️ BlackLake CLI

### CLI Service

The CLI service provides an interactive shell with the BlackLake CLI pre-installed.

### Using the CLI

#### Start CLI Service
```bash
# Start CLI service
docker-compose up cli

# Or run interactively
docker-compose run --rm cli
```

#### CLI Commands
Once in the CLI container, you can use all BlackLake CLI commands:

```bash
# List repositories
blacklake-cli repos list

# Search for files
blacklake-cli search --query "documentation"

# Upload a file
blacklake-cli put --file /data/myfile.txt --repo my-repo

# Get repository info
blacklake-cli repos get my-repo
```

### CLI Environment

The CLI service is configured with:

- **Working Directory**: `/data` (mounted from `./data`)
- **Database Access**: Connected to the main database
- **S3 Access**: Connected to MinIO
- **API Access**: Connected to the API service

### Volume Mounts

The CLI service mounts:
- `./data:/data` - Your local data directory

## 🚀 Quick Start

### 1. Start the Full Stack
```bash
# Start all services with migrations
docker-compose up -d

# Check migration status
docker-compose logs migrations
```

### 2. Use the CLI
```bash
# Start CLI service
docker-compose up cli

# Or run a one-off command
docker-compose run --rm cli blacklake-cli repos list
```

### 3. Verify Setup
```bash
# Check all services are running
docker-compose ps

# Check database schema
docker-compose exec db psql -U blacklake -d blacklake -c "\dt"
```

## 🔧 Configuration

### Environment Variables

The migration and CLI services use these environment variables:

```bash
# Database
DATABASE_URL=postgresql://blacklake:blacklake@db:5432/blacklake

# S3/MinIO
S3_ENDPOINT=http://minio:9000
S3_ACCESS_KEY_ID=minioadmin
S3_SECRET_ACCESS_KEY=minioadmin
S3_BUCKET=blacklake
S3_REGION=us-east-1

# API
API_BASE_URL=http://api:8080

# Logging
RUST_LOG=info
```

### Custom Migration Scripts

To add custom migrations:

1. Create a new SQL file in `migrations/` with the next number
2. Update `scripts/run-migrations.sh` to include your migration
3. The migration will run automatically on next startup

## 🐛 Troubleshooting

### Migration Issues

```bash
# Check migration logs
docker-compose logs migrations

# Run migrations manually
docker-compose run --rm migrations

# Check database connection
docker-compose exec db psql -U blacklake -d blacklake -c "SELECT version();"
```

### CLI Issues

```bash
# Check CLI logs
docker-compose logs cli

# Test CLI connection
docker-compose run --rm cli blacklake-cli --help

# Check environment variables
docker-compose run --rm cli env | grep -E "(DATABASE_URL|S3_|API_)"
```

### Database Schema Issues

```bash
# Check current schema
docker-compose exec db psql -U blacklake -d blacklake -c "\dt"

# Reset database (WARNING: This will delete all data)
docker-compose down -v
docker-compose up -d db
docker-compose run --rm migrations
```

## 📚 Additional Resources

- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [SQLx Migration Guide](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md)
- [BlackLake CLI Documentation](cli.md)
