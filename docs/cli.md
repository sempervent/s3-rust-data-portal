# BlackLake CLI Documentation

The BlackLake CLI provides a comprehensive command-line interface for managing data artifacts, repositories, and performing various operations with Git-like version control capabilities.

## Installation

The CLI is included in the Docker Compose setup and can be used directly:

```bash
# Start CLI service
docker-compose up cli

# Or run interactively
docker-compose run --rm cli

# Install locally (if building from source)
cargo install --path crates/cli
```

## Repository Initialization

### Initialize a New Repository

The `blacklake-cli init` command initializes a directory as a BlackLake repository with comprehensive metadata support:

```bash
# Basic initialization
blacklake-cli init

# Initialize with specific name
blacklake-cli init --name "my-ml-project"

# Initialize with comprehensive metadata
blacklake-cli init \
  --name "advanced-ml-project" \
  --description "Advanced Machine Learning Pipeline with MLOps" \
  --author "Data Science Team" \
  --license "MIT" \
  --tags "ml,ai,pipeline,production" \
  --version "1.0.0"
```

### Metadata Configuration with Dot Notation

The CLI supports dot notation for setting nested metadata values:

```bash
# Set basic metadata
blacklake-cli init \
  --set repository.name="my-project" \
  --set repository.description="Machine Learning Pipeline" \
  --set repository.author="Data Science Team" \
  --set repository.license="MIT" \
  --set repository.version="1.0.0"

# Set environment metadata
blacklake-cli init \
  --set environment.os="linux" \
  --set environment.python_version="3.11" \
  --set environment.r_version="4.3" \
  --set environment.julia_version="1.9"

# Set compliance metadata
blacklake-cli init \
  --set compliance.data_classification="internal" \
  --set compliance.retention_policy="7y" \
  --set compliance.access_control="team" \
  --set compliance.audit_logging=true

# Set workflow metadata
blacklake-cli init \
  --set workflow.stages="data-preprocessing,model-training,evaluation" \
  --set workflow.pipeline="scikit-learn,tensorflow,mlflow" \
  --set workflow.artifacts="models,datasets,reports"

# Set dependencies
blacklake-cli init \
  --set dependencies.python="pandas,numpy,scikit-learn,tensorflow" \
  --set dependencies.r="dplyr,ggplot2,caret" \
  --set dependencies.julia="Flux.jl,MLJ.jl" \
  --set dependencies.other="docker,kubernetes"
```

### Generated Structure

After initialization, you'll have:

```
my-project/
├── .bl/                          # BlackLake metadata directory (like .git)
│   ├── config.toml              # Repository configuration
│   ├── metadata.json            # Repository metadata
│   ├── .gitignore              # BlackLake-specific gitignore
│   ├── hooks/                  # BlackLake hooks
│   │   ├── pre-commit
│   │   ├── post-commit
│   │   └── pre-push
│   ├── refs/                   # Reference tracking
│   │   ├── main
│   │   └── tags/
│   ├── objects/                # Object storage
│   │   ├── blobs/
│   │   └── commits/
│   └── index                   # Staging area
├── data/                       # Your data files
├── models/                     # ML models
├── datasets/                   # Training datasets
├── notebooks/                  # Jupyter notebooks
└── README.md                   # Project documentation
```

### Configuration File

The `.bl/config.toml` file contains:

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

[hooks]
pre_commit = "blacklake-cli hooks pre-commit"
post_commit = "blacklake-cli hooks post-commit"
pre_push = "blacklake-cli hooks pre-push"
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

## Repository Management

### Repository Operations

```bash
# List repositories
blacklake-cli repos list

# Get repository information
blacklake-cli repos get <repo-name>

# Create a new repository
blacklake-cli repos create --name "my-repo" --description "My ML Repository"

# Update repository metadata
blacklake-cli repos update <repo-name> --set repository.description="Updated description"

# Delete repository
blacklake-cli repos delete <repo-name>
```

### Repository Status

```bash
# Check repository status
blacklake-cli status

# Show repository information
blacklake-cli info

# Show repository history
blacklake-cli log

# Show repository branches
blacklake-cli branch

# Show repository tags
blacklake-cli tag
```

## File Operations

### Adding Files

```bash
# Add a single file
blacklake-cli add data/myfile.txt

# Add multiple files
blacklake-cli add data/*.txt

# Add entire directory
blacklake-cli add data/

# Add with metadata
blacklake-cli add data/model.onnx --set metadata.type="onnx" --set metadata.version="1.0"
```

### File Management

```bash
# List files in repository
blacklake-cli ls

# List files with details
blacklake-cli ls --long

# Show file information
blacklake-cli show data/myfile.txt

# Download a file
blacklake-cli get data/myfile.txt

# Remove a file
blacklake-cli rm data/myfile.txt
```

### Version Control

```bash
# Commit changes
blacklake-cli commit -m "Initial commit"

# Commit with metadata
blacklake-cli commit -m "Added model" --set commit.author="Data Scientist" --set commit.tags="model,v1.0"

# Show commit history
blacklake-cli log

# Show specific commit
blacklake-cli show <commit-hash>

# Create a tag
blacklake-cli tag v1.0.0 -m "Release version 1.0.0"

# List tags
blacklake-cli tag --list
```

## Search Operations

### Basic Search

```bash
# Search for files
blacklake-cli search --query "documentation"

# Search by metadata
blacklake-cli search --metadata "tags:ml,ai"

# Search by file type
blacklake-cli search --type onnx

# Search with filters
blacklake-cli search --query "model" --type onnx --limit 10
```

### Advanced Search

```bash
# Search by author
blacklake-cli search --author "Data Scientist"

# Search by date range
blacklake-cli search --since "2024-01-01" --until "2024-12-31"

# Search by size
blacklake-cli search --size ">100MB"

# Search by content
blacklake-cli search --content "machine learning"

# Search with complex metadata
blacklake-cli search --metadata "compliance.data_classification=internal" --metadata "workflow.stages=training"
```

## Configuration

### Environment Variables

```bash
# Set API endpoint
export BLACKLAKE_API_URL=http://localhost:8080

# Set authentication token
export BLACKLAKE_TOKEN=your-jwt-token

# Set default repository
export BLACKLAKE_DEFAULT_REPO=my-repo

# Set storage backend
export BLACKLAKE_STORAGE_BACKEND=s3
export BLACKLAKE_S3_BUCKET=blacklake
export BLACKLAKE_S3_REGION=us-east-1

# Set search backend
export BLACKLAKE_SEARCH_BACKEND=postgres
export BLACKLAKE_DATABASE_URL=postgresql://user:pass@localhost/blacklake

# Set logging level
export RUST_LOG=info
```

### Configuration File

Create a `~/.blacklake/config.toml` file:

```toml
[api]
url = "http://localhost:8080"
token = "your-jwt-token"

[storage]
backend = "s3"
bucket = "blacklake"
region = "us-east-1"

[search]
backend = "postgres"
database_url = "postgresql://user:pass@localhost/blacklake"

[auth]
provider = "oidc"
issuer = "https://keycloak.example.com/realms/blacklake"

[logging]
level = "info"
```

## Advanced Usage

### Batch Operations

```bash
# Upload multiple files
blacklake-cli put --file /data/*.txt --repo my-repo

# Batch add with metadata
blacklake-cli add data/ --set metadata.batch="true" --set metadata.date="2024-01-15"

# Batch commit
blacklake-cli commit -m "Batch update" --set commit.batch="true"
```

### Hooks and Automation

```bash
# List available hooks
blacklake-cli hooks --list

# Install hooks
blacklake-cli hooks install

# Run specific hook
blacklake-cli hooks pre-commit

# Custom hook example
blacklake-cli hooks add pre-commit "blacklake-cli validate"
```

### Export and Import

```bash
# Export repository
blacklake-cli export --format tar.gz --output my-repo.tar.gz

# Import repository
blacklake-cli import my-repo.tar.gz

# Export metadata only
blacklake-cli export --metadata-only --output metadata.json
```

## Troubleshooting

### Common Issues

1. **Authentication errors**: Ensure you have a valid JWT token
2. **Connection errors**: Check that the API service is running
3. **File not found**: Verify the file path and repository name
4. **Permission errors**: Check file permissions and repository access
5. **Metadata errors**: Validate metadata format and required fields

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
blacklake-cli --help

# Verbose output
blacklake-cli --verbose <command>

# Dry run mode
blacklake-cli --dry-run <command>
```

### Logging

```bash
# Set log level
export RUST_LOG=debug

# Log to file
blacklake-cli --log-file /tmp/blacklake.log <command>

# Show logs
blacklake-cli logs

# Clear logs
blacklake-cli logs --clear
```

## Command Reference

### Global Options

```bash
blacklake-cli [OPTIONS] <COMMAND>

Options:
  -v, --verbose              Verbose output
  -q, --quiet                Quiet output
  -h, --help                 Print help
  -V, --version              Print version
  --config <CONFIG>          Config file path
  --log-level <LEVEL>        Log level (error, warn, info, debug, trace)
  --dry-run                  Show what would be done without executing
```

### Repository Commands

```bash
blacklake-cli repos <COMMAND>

Commands:
  list                       List repositories
  get <name>                Get repository information
  create                    Create new repository
  update <name>             Update repository
  delete <name>             Delete repository
  clone <url>               Clone repository
  remote                    Manage remotes
```

### File Commands

```bash
blacklake-cli <COMMAND>

Commands:
  add <path>                Add files to staging
  rm <path>                 Remove files
  mv <src> <dst>            Move/rename files
  cp <src> <dst>            Copy files
  ls                        List files
  show <path>               Show file information
  get <path>                Download file
```

### Version Control Commands

```bash
blacklake-cli <COMMAND>

Commands:
  commit                    Commit changes
  log                       Show commit history
  show <hash>               Show commit details
  branch                    Manage branches
  tag                       Manage tags
  diff                      Show differences
  status                    Show repository status
```

## Additional Resources

- [API Documentation](https://github.com/NAERM/s3-rust-data-portal/tree/main/crates/api)
- [Getting Started Guide](getting-started.md)
- [Local Testing Guide](local_testing.md)
- [Migration Setup](MIGRATION_SETUP.md)
- [Project Status](PROJECT_STATUS.md)
