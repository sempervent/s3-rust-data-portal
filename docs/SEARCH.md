# Search in BlackLake

BlackLake provides powerful search capabilities to help you discover and explore your data. This guide covers all aspects of search functionality.

## Overview

BlackLake's search is built on Apache Solr and provides:

- **Full-text Search**: Search across file contents and metadata
- **Faceted Search**: Filter results by file type, size, date, etc.
- **Semantic Search**: AI-powered semantic understanding
- **Suggestions**: Auto-complete and search suggestions
- **Saved Searches**: Save and share search queries
- **Real-time Indexing**: Search results update immediately

## Basic Search

### Simple Text Search

The most basic search is a simple text query:

```bash
# CLI
blacklake search "machine learning"

# API
GET /api/v1/search?q=machine%20learning
```

### Search Operators

BlackLake supports advanced search operators:

#### Boolean Operators
```bash
# AND search (both terms must be present)
blacklake search "machine AND learning"

# OR search (either term can be present)
blacklake search "machine OR learning"

# NOT search (exclude terms)
blacklake search "machine NOT python"
```

#### Phrase Search
```bash
# Exact phrase matching
blacklake search '"machine learning"'

# Proximity search (terms within N words)
blacklake search '"machine learning"~5'
```

#### Wildcard Search
```bash
# Single character wildcard
blacklake search "mod?l"

# Multiple character wildcard
blacklake search "model*"
```

#### Fuzzy Search
```bash
# Fuzzy matching (handles typos)
blacklake search "machne~"  # matches "machine"
```

## Advanced Search

### Field-Specific Search

Search within specific metadata fields:

```bash
# Search in file name
blacklake search "name:model.pkl"

# Search in file type
blacklake search "type:csv"

# Search in description
blacklake search "description:training data"
```

### Range Queries

Search within ranges:

```bash
# File size range
blacklake search "size:[1000000 TO 5000000]"

# Date range
blacklake search "created:[2024-01-01 TO 2024-12-31]"

# Numeric range
blacklake search "version:[1.0 TO 2.0]"
```

### Complex Queries

Combine multiple conditions:

```bash
# Complex boolean query
blacklake search "(machine learning OR deep learning) AND (python OR tensorflow) AND size:[1000000 TO *]"

# Field-specific with ranges
blacklake search "type:csv AND size:[1000000 TO *] AND created:[2024-01-01 TO *]"
```

## Faceted Search

Faceted search allows you to filter results by categories:

### Available Facets

- **File Type**: Filter by MIME type (csv, json, pkl, etc.)
- **Size**: Filter by file size ranges
- **Date**: Filter by creation/modification date
- **Repository**: Filter by repository
- **Tags**: Filter by user-defined tags
- **Owner**: Filter by file owner

### Using Facets

#### CLI
```bash
# Filter by file type
blacklake search "data" --type csv

# Filter by size
blacklake search "model" --size ">1MB"

# Filter by date
blacklake search "dataset" --created ">2024-01-01"

# Multiple filters
blacklake search "training" --type csv --size ">1MB" --created ">2024-01-01"
```

#### API
```bash
# Faceted search via API
GET /api/v1/search?q=data&facet=type:csv&facet=size:>1MB&facet=created:>2024-01-01
```

#### Web UI
1. Enter your search query
2. Use the facet filters on the left sidebar
3. Click on facet values to apply filters
4. Use the "Clear Filters" button to reset

### Facet Statistics

Get statistics about facet values:

```bash
# Get file type distribution
blacklake search "data" --facets type

# Get size distribution
blacklake search "model" --facets size

# Get date distribution
blacklake search "dataset" --facets created
```

## Semantic Search

BlackLake includes AI-powered semantic search capabilities:

### How It Works

1. **Embeddings**: Files are processed to generate vector embeddings
2. **Vector Search**: Queries are converted to embeddings and matched against file embeddings
3. **Ranking**: Results are ranked by semantic similarity
4. **Hybrid Search**: Combines traditional text search with semantic search

### Using Semantic Search

#### CLI
```bash
# Semantic search
blacklake search "find files related to image classification" --semantic

# Hybrid search (combines text and semantic)
blacklake search "machine learning models" --hybrid
```

#### API
```bash
# Semantic search via API
GET /api/v1/search?q=image%20classification&semantic=true

# Hybrid search
GET /api/v1/search?q=machine%20learning&hybrid=true
```

#### Web UI
1. Enable "Semantic Search" in the search options
2. Enter your query in natural language
3. Results will be ranked by semantic similarity

### Semantic Search Examples

```bash
# Find files related to a concept
blacklake search "files about data preprocessing" --semantic

# Find similar files
blacklake search "files similar to model.pkl" --semantic

# Find files by use case
blacklake search "files for training neural networks" --semantic
```

## Search Suggestions

BlackLake provides intelligent search suggestions:

### Auto-complete

As you type, BlackLake suggests:

- **File names**: Matching file names
- **Tags**: User-defined tags
- **Queries**: Previously used search queries
- **Concepts**: AI-suggested concepts

### Using Suggestions

#### CLI
```bash
# Get suggestions for a partial query
blacklake search --suggest "mach"

# Get suggestions for a specific field
blacklake search --suggest "name:mach"
```

#### API
```bash
# Get suggestions via API
GET /api/v1/search/suggest?q=mach

# Get field-specific suggestions
GET /api/v1/search/suggest?q=mach&field=name
```

#### Web UI
- Suggestions appear as you type in the search box
- Click on suggestions to complete your query
- Use arrow keys to navigate suggestions

## Saved Searches

Save frequently used searches for quick access:

### Creating Saved Searches

#### CLI
```bash
# Save a search
blacklake search "machine learning" --save "ml-files"

# Save with description
blacklake search "type:csv AND size:>1MB" --save "large-csv-files" --description "Large CSV files"
```

#### Web UI
1. Perform your search
2. Click "Save Search" button
3. Enter a name and description
4. Choose visibility (private or shared)

### Using Saved Searches

#### CLI
```bash
# List saved searches
blacklake search --list-saved

# Use a saved search
blacklake search --saved "ml-files"

# Update a saved search
blacklake search "machine learning AND python" --update "ml-files"
```

#### Web UI
1. Click on "Saved Searches" in the sidebar
2. Click on a saved search to execute it
3. Edit or delete saved searches from the management interface

### Sharing Saved Searches

```bash
# Share a saved search
blacklake search --share "ml-files" --with "team-member@example.com"

# Make a saved search public
blacklake search --make-public "ml-files"
```

## Search Configuration

### Search Settings

Configure search behavior:

#### CLI
```bash
# Set default search options
blacklake config set search.default-type semantic
blacklake config set search.results-per-page 50
blacklake config set search.highlight true
```

#### Configuration File
```toml
[search]
default_type = "semantic"
results_per_page = 50
highlight = true
facets = ["type", "size", "created", "tags"]
suggestions = true
```

### Search Indexing

#### Index Management
```bash
# Reindex a repository
blacklake search --reindex my-repo

# Reindex all repositories
blacklake search --reindex-all

# Check index status
blacklake search --status
```

#### Index Optimization
```bash
# Optimize search index
blacklake search --optimize

# Check index health
blacklake search --health
```

## Performance Optimization

### Search Performance

#### Query Optimization
- **Use specific fields**: Search specific fields instead of all fields
- **Limit results**: Use pagination to limit result sets
- **Use filters**: Apply filters to reduce search scope
- **Cache queries**: Enable query caching for repeated searches

#### Index Optimization
- **Regular reindexing**: Keep indexes up to date
- **Index optimization**: Periodically optimize indexes
- **Field selection**: Only index necessary fields
- **Stop words**: Configure appropriate stop words

### Monitoring Search Performance

#### Metrics
- **Query time**: Average query response time
- **Index size**: Search index size
- **Cache hit rate**: Query cache effectiveness
- **Error rate**: Search error rate

#### Monitoring Commands
```bash
# Get search statistics
blacklake search --stats

# Monitor search performance
blacklake search --monitor

# Check search health
blacklake search --health
```

## Search API Reference

### Endpoints

#### Basic Search
```http
GET /api/v1/search?q={query}
```

#### Faceted Search
```http
GET /api/v1/search?q={query}&facet={field}:{value}
```

#### Semantic Search
```http
GET /api/v1/search?q={query}&semantic=true
```

#### Search Suggestions
```http
GET /api/v1/search/suggest?q={query}
```

#### Saved Searches
```http
GET /api/v1/search/saved
POST /api/v1/search/saved
PUT /api/v1/search/saved/{id}
DELETE /api/v1/search/saved/{id}
```

### Response Format

```json
{
  "query": "machine learning",
  "total_results": 150,
  "results": [
    {
      "id": "file-123",
      "name": "model.pkl",
      "path": "/models/model.pkl",
      "type": "application/octet-stream",
      "size": 2048576,
      "created": "2024-01-15T10:30:00Z",
      "modified": "2024-01-15T10:30:00Z",
      "metadata": {
        "description": "Machine learning model",
        "tags": ["ml", "model", "python"]
      },
      "highlights": {
        "content": "This is a <em>machine learning</em> model"
      }
    }
  ],
  "facets": {
    "type": {
      "csv": 45,
      "json": 30,
      "pkl": 25
    },
    "size": {
      "<1MB": 60,
      "1MB-10MB": 70,
      ">10MB": 20
    }
  },
  "suggestions": [
    "machine learning",
    "machine learning models",
    "machine learning algorithms"
  ]
}
```

## Best Practices

### Search Query Design

1. **Start Simple**: Begin with basic text search
2. **Add Filters**: Use facets to narrow results
3. **Use Operators**: Leverage boolean operators for complex queries
4. **Test Queries**: Validate search queries before saving
5. **Monitor Performance**: Watch for slow queries

### Search Index Management

1. **Regular Updates**: Keep search indexes current
2. **Optimize Periodically**: Run index optimization
3. **Monitor Size**: Watch index size growth
4. **Backup Indexes**: Include search indexes in backups
5. **Test Restores**: Verify search index restoration

### User Experience

1. **Provide Suggestions**: Enable auto-complete
2. **Show Facets**: Display available filters
3. **Highlight Results**: Highlight matching terms
4. **Save Searches**: Allow saving frequent queries
5. **Share Results**: Enable sharing of search results

This comprehensive search functionality makes BlackLake a powerful platform for data discovery and exploration. The combination of traditional text search, faceted filtering, and semantic search provides users with multiple ways to find the data they need.