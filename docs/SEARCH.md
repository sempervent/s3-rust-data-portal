# BlackLake Search Architecture

## Overview

BlackLake uses Apache Solr for advanced search capabilities, providing full-text search, faceting, suggestions, and complex query support for data artifacts.

## Architecture

### SolrCloud Setup

- **Single-node SolrCloud** with embedded ZooKeeper for development
- **Collection**: `blacklake` with 1 shard, 1 replica
- **Configset**: `blacklake` with managed schema and custom analyzers

### Search Features

#### Core Search Fields

- `file_name`: Full-text searchable file names
- `title`: Artifact titles with English stemming
- `description`: Rich text descriptions with advanced analysis
- `tags`: Multi-valued tags for categorization
- `org_lab`: Organization/laboratory identifiers
- `file_type`: MIME types and file extensions
- `file_size`: Numeric file sizes for range queries
- `creation_dt`: Creation timestamps for date faceting
- `path`: Repository paths for hierarchical navigation
- `repo_id`, `commit_id`: Repository and version identifiers

#### Text Analysis

- **English Stemming**: Porter stemmer for `description` field
- **Synonyms**: Configurable synonym expansion via `synonyms.txt`
- **Stopwords**: Minimal English stopword filtering
- **Case Insensitive**: All text fields normalized to lowercase

#### Advanced Features

- **JSON Facet API**: Dynamic faceting on tags, file_type, org_lab, creation_dt
- **Suggester**: Typeahead suggestions for file names and tags
- **Export Handler**: Large result set export for bulk operations
- **Near Real-time**: Soft commits every 1.5s for fresh results

## Indexing Strategy

### Event-Driven Indexing

```rust
// Triggered on commit.created events
IndexEntryJob {
    repo_id: Uuid,
    commit_id: Uuid,
    path: String,
    object_sha256: Option<String>,
    meta: serde_json::Value,
}
```

### Document Structure

```json
{
  "id": "repo_id:commit_id:path",
  "repo_id": "uuid",
  "commit_id": "uuid", 
  "path": "data/experiments/2024/",
  "file_name": "experiment_results.csv",
  "title": "Q4 2024 Experiment Results",
  "description": "Machine learning experiment results...",
  "tags": ["ml", "experiment", "q4-2024"],
  "org_lab": "ornl",
  "file_type": "text/csv",
  "file_size": 1048576,
  "creation_dt": "2024-01-15T10:30:00Z",
  "object_sha256": "abc123...",
  "_version_": 1
}
```

### Durability & Consistency

- **Optimistic Concurrency**: `_version_` field prevents lost updates
- **Commit Strategy**: `commitWithin=1500ms` for soft commits
- **Hard Commits**: Batched by Apalis workers every 15s or 10k docs
- **Alias Management**: `latest` alias per path for current versions

## Query API

### Basic Search

```http
GET /v1/search?q=machine learning&limit=20&offset=0
```

### Advanced Search

```http
GET /v1/search?q=*:*&fq=tags:ml&fq=org_lab:ornl&sort=creation_dt desc
```

### Faceted Search

```http
GET /v1/search?q=*:*&json.facet={
  "tags": {"type": "terms", "field": "tags", "limit": 10},
  "file_types": {"type": "terms", "field": "file_type", "limit": 5},
  "date_histogram": {
    "type": "range", 
    "field": "creation_dt",
    "ranges": [{"from": "2024-01-01", "to": "2024-12-31"}]
  }
}
```

### Suggestions

```http
GET /v1/search/suggest?q=mach&count=5
```

## Performance Tuning

### Indexing Performance

- **Batch Size**: 1000 documents per batch
- **Concurrency**: 4 parallel indexing workers
- **Memory**: 512MB heap for Solr
- **Storage**: SSD recommended for index storage

### Query Performance

- **Caching**: Query result cache (512MB)
- **Filter Cache**: 256MB for common filters
- **Field Cache**: For sorting and faceting
- **Warmup**: Auto-warm queries on startup

### Monitoring

- **Search Latency**: P50, P95, P99 percentiles
- **Index Size**: Document count and disk usage
- **Commit Frequency**: Soft/hard commit timing
- **Cache Hit Rates**: Query and filter cache efficiency

## Configuration

### Environment Variables

```bash
SOLR_URL=http://solr:8983
SOLR_COLLECTION=blacklake
SOLR_COMMIT_WITHIN=1500
SOLR_BATCH_SIZE=1000
```

### Schema Management

- **Managed Schema**: Dynamic field additions via API
- **Field Types**: Predefined analyzers for different content types
- **Copy Fields**: Automatic text field population
- **Dynamic Fields**: Flexible metadata indexing

## Troubleshooting

### Common Issues

1. **Slow Queries**: Check query complexity and use filters
2. **Memory Issues**: Increase heap size or reduce batch size
3. **Index Corruption**: Reindex from scratch using `/v1/search/reindex`
4. **ZooKeeper Issues**: Restart Solr service

### Health Checks

```bash
# Check Solr health
curl http://localhost:8983/solr/admin/ping

# Check collection status
curl http://localhost:8983/solr/admin/collections?action=STATUS&collection=blacklake

# Check index statistics
curl http://localhost:8983/solr/blacklake/admin/mbeans?stats=true
```

## Migration from OpenSearch

### Data Migration

1. **Export**: Use OpenSearch scroll API to export all documents
2. **Transform**: Convert OpenSearch document format to Solr schema
3. **Import**: Bulk import to Solr using `/update/json/docs`
4. **Verify**: Compare document counts and sample queries

### Query Migration

- **Query Syntax**: Convert OpenSearch query DSL to Solr query syntax
- **Facets**: Migrate from OpenSearch aggregations to Solr JSON Facet API
- **Sorting**: Update sort field names and syntax
- **Pagination**: Adjust offset/limit parameters

## Future Enhancements

### Planned Features

- **Multi-language Support**: Additional analyzers for different languages
- **Semantic Search**: Vector similarity search for content discovery
- **Auto-complete**: Real-time search suggestions
- **Search Analytics**: Query performance and user behavior tracking
- **Distributed Search**: Multi-node SolrCloud for production scaling
