# BlackLake CLI Documentation

The BlackLake CLI provides a command-line interface for managing data artifacts, repositories, and performing various operations.

## Installation

The CLI is included in the Docker Compose setup and can be used directly:

```bash
# Start CLI service
docker-compose up cli

# Or run interactively
docker-compose run --rm cli
```

## Basic Commands

### Repository Management

```bash
# List repositories
blacklake-cli repos list

# Get repository information
blacklake-cli repos get <repo-name>

# Create a new repository
blacklake-cli repos create --name "my-repo" --description "My ML Repository"
```

### File Operations

```bash
# Upload a file
blacklake-cli put --file /data/myfile.txt --repo my-repo

# Download a file
blacklake-cli get --repo my-repo --path data/myfile.txt

# List files in repository
blacklake-cli ls --repo my-repo
```

### Search Operations

```bash
# Search for files
blacklake-cli search --query "documentation"

# Search by metadata
blacklake-cli search --metadata "tags:ml,ai"

# Search by file type
blacklake-cli search --type onnx
```

## Advanced Usage

### Configuration

The CLI can be configured using environment variables or a configuration file:

```bash
# Set API endpoint
export BLACKLAKE_API_URL=http://localhost:8080

# Set authentication token
export BLACKLAKE_TOKEN=your-jwt-token

# Set default repository
export BLACKLAKE_DEFAULT_REPO=my-repo
```

### Batch Operations

```bash
# Upload multiple files
blacklake-cli put --file /data/*.txt --repo my-repo

# Search with filters
blacklake-cli search --query "model" --type onnx --limit 10
```

## Troubleshooting

### Common Issues

1. **Authentication errors**: Ensure you have a valid JWT token
2. **Connection errors**: Check that the API service is running
3. **File not found**: Verify the file path and repository name

### Debug Mode

```bash
# Enable debug logging
export RUST_LOG=debug
blacklake-cli --help
```

## Additional Resources

- [API Documentation](api/)
- [Getting Started Guide](getting-started.md)
- [Local Testing Guide](local_testing.md)
