# Getting Started with BlackLake

Welcome to BlackLake, a modern data platform designed for machine learning and data science workflows. This guide will help you get up and running quickly.

## What is BlackLake?

BlackLake is a comprehensive data platform that provides:

- **Version Control for Data**: Git-like versioning for datasets and models
- **Metadata Management**: Rich metadata extraction and indexing
- **Search & Discovery**: Powerful search capabilities with faceted filtering
- **Compliance & Governance**: Built-in data governance and compliance features
- **Multi-tenant Architecture**: Secure, isolated workspaces for teams
- **API-First Design**: RESTful APIs and SDKs for integration

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Git
- Node.js 18+ (for UI development)
- Rust 1.70+ (for API development)

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-org/blacklake.git
   cd blacklake
   ```

2. **Start the development environment**
   ```bash
   docker-compose up -d
   ```

3. **Verify installation**
   ```bash
   curl http://localhost:8080/health
   ```

4. **Access the web UI**
   Open http://localhost:3000 in your browser

### First Steps

1. **Create your first repository**
   ```bash
   # Using the CLI
   blacklake repo create my-first-repo
   
   # Or via the web UI
   # Navigate to the "Repositories" section and click "New Repository"
   ```

2. **Upload your first dataset**
   ```bash
   # Using the CLI
   blacklake upload my-first-repo ./data/my-dataset.csv
   
   # Or via the web UI
   # Navigate to your repository and click "Upload Files"
   ```

3. **Search for your data**
   ```bash
   # Using the CLI
   blacklake search "machine learning"
   
   # Or via the web UI
   # Use the search bar at the top of the interface
   ```

## Core Concepts

### Repositories

Repositories are the primary organizational unit in BlackLake. They contain:

- **Files**: Your datasets, models, and other artifacts
- **Metadata**: Automatically extracted information about your files
- **History**: Complete version history of all changes
- **Permissions**: Access control for team members

### Entries

Entries represent individual files within a repository. Each entry includes:

- **Content**: The actual file data
- **Metadata**: Automatically extracted information
- **Lineage**: Relationships to other entries
- **Versions**: Complete change history

### Metadata

BlackLake automatically extracts rich metadata from your files:

- **Technical Metadata**: File size, type, checksums
- **Content Metadata**: Schema, statistics, previews
- **ML Metadata**: Model architecture, training parameters
- **Custom Metadata**: User-defined tags and properties

## Authentication

BlackLake supports multiple authentication methods:

### OIDC/JWT (Recommended)
```bash
# Set your JWT token
export BLACKLAKE_TOKEN="your-jwt-token"

# Or use the CLI login command
blacklake auth login
```

### API Keys
```bash
# Generate an API key via the web UI
# Settings > API Keys > Generate New Key

# Use the API key
export BLACKLAKE_API_KEY="your-api-key"
```

## CLI Usage

The BlackLake CLI provides powerful command-line access to all features:

### Repository Management
```bash
# List repositories
blacklake repo list

# Create a new repository
blacklake repo create my-repo --description "My data repository"

# Get repository information
blacklake repo info my-repo
```

### File Operations
```bash
# Upload files
blacklake upload my-repo ./data/file.csv

# Download files
blacklake download my-repo file.csv

# List files in a repository
blacklake ls my-repo
```

### Search
```bash
# Search across all repositories
blacklake search "machine learning"

# Search within a specific repository
blacklake search "model" --repo my-repo

# Advanced search with filters
blacklake search "data" --type csv --size ">1MB"
```

### Version Control
```bash
# Commit changes
blacklake commit my-repo -m "Add new dataset"

# View history
blacklake log my-repo

# Create branches
blacklake branch my-repo create feature-branch
```

## Web UI

The BlackLake web interface provides a modern, intuitive experience:

### Dashboard
- Overview of your repositories
- Recent activity and changes
- Quick access to common tasks

### Repository View
- File browser with preview capabilities
- Metadata display and editing
- Version history and branching

### Search Interface
- Powerful search with faceted filtering
- Real-time suggestions and autocomplete
- Saved searches and bookmarks

### Admin Panel
- User and permission management
- System configuration
- Monitoring and analytics

## API Integration

BlackLake provides comprehensive REST APIs for integration:

### Authentication
```bash
# Include your token in requests
curl -H "Authorization: Bearer $BLACKLAKE_TOKEN" \
     http://localhost:8080/api/v1/repos
```

### Basic Operations
```python
import requests

# List repositories
response = requests.get(
    "http://localhost:8080/api/v1/repos",
    headers={"Authorization": f"Bearer {token}"}
)
repos = response.json()

# Upload a file
with open("data.csv", "rb") as f:
    response = requests.post(
        f"http://localhost:8080/api/v1/repos/{repo_id}/files/data.csv",
        headers={"Authorization": f"Bearer {token}"},
        data=f
    )
```

### Python SDK
```python
from blacklake import BlackLakeClient

client = BlackLakeClient(token="your-token")

# Create a repository
repo = client.repos.create("my-repo", description="My repository")

# Upload a file
client.files.upload(repo.id, "data.csv", "path/to/data.csv")

# Search for files
results = client.search("machine learning")
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `BLACKLAKE_API_URL` | API server URL | `http://localhost:8080` |
| `BLACKLAKE_TOKEN` | Authentication token | - |
| `BLACKLAKE_API_KEY` | API key for authentication | - |
| `BLACKLAKE_DEBUG` | Enable debug logging | `false` |

### Configuration File

Create a `~/.blacklake/config.toml` file:

```toml
[api]
url = "http://localhost:8080"
timeout = 30

[auth]
token = "your-jwt-token"
# or
api_key = "your-api-key"

[logging]
level = "info"
format = "json"
```

## Next Steps

Now that you have BlackLake running, explore these advanced features:

1. **Team Collaboration**: Set up user permissions and team workspaces
2. **Advanced Search**: Learn about faceted search and saved queries
3. **Metadata Management**: Customize metadata extraction and schemas
4. **Compliance**: Configure retention policies and audit logging
5. **Integration**: Connect with your existing ML pipelines

## Getting Help

- **Documentation**: Browse the complete documentation
- **Community**: Join our community forum
- **Support**: Contact support for enterprise features
- **GitHub**: Report issues and contribute

## Examples

### Data Science Workflow
```bash
# 1. Create a repository for your project
blacklake repo create ml-project

# 2. Upload your training data
blacklake upload ml-project ./data/train.csv
blacklake upload ml-project ./data/test.csv

# 3. Upload your trained model
blacklake upload ml-project ./models/model.pkl

# 4. Commit your work
blacklake commit ml-project -m "Initial dataset and model"

# 5. Search for your work later
blacklake search "model" --repo ml-project
```

### Team Collaboration
```bash
# 1. Create a team repository
blacklake repo create team-data --public

# 2. Add team members (via web UI)
# Navigate to repository settings and add users

# 3. Set up branch protection
blacklake branch team-data protect main

# 4. Work on feature branches
blacklake branch team-data create feature/new-dataset
blacklake upload team-data ./new-data.csv
blacklake commit team-data -m "Add new dataset"
```

This completes your introduction to BlackLake. You're now ready to start using the platform for your data science and machine learning workflows!
