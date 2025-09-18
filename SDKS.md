# BlackLake SDKs

## Overview

BlackLake provides official SDKs for Python and TypeScript to simplify integration with the data portal API.

## Python SDK

### Installation

```bash
# From PyPI
pip install blacklake-sdk

# From source
pip install git+https://github.com/blacklake/blacklake.git#subdirectory=sdks/python
```

### Basic Usage

```python
import asyncio
from blacklake import BlackLakeClient

async def main():
    # Initialize client
    client = BlackLakeClient(
        base_url="http://localhost:8080",
        api_key="your-api-key"
    )
    
    # List repositories
    repos = await client.list_repositories()
    print(f"Found {len(repos)} repositories")
    
    # Search files
    results = await client.search("sales data", limit=10)
    print(f"Found {results.total} results")
    
    # Get repository tree
    tree = await client.get_repository_tree("my-repo", "main")
    for entry in tree.data:
        print(f"{entry.type}: {entry.path}")
    
    await client.close()

# Run async code
asyncio.run(main())
```

### Synchronous Usage

```python
from blacklake import BlackLakeClientSync

# Use synchronous wrapper
with BlackLakeClientSync(base_url="http://localhost:8080") as client:
    repos = client.list_repositories()
    results = client.search("sales data")
    tree = client.get_repository_tree("my-repo")
```

### File Upload

```python
async def upload_file(client, repo_name, file_path):
    # Get file info
    file_size = os.path.getsize(file_path)
    content_type = mimetypes.guess_type(file_path)[0]
    
    # Initiate upload
    upload_info = await client.initiate_upload(
        repo_name=repo_name,
        path=file_path,
        size=file_size,
        content_type=content_type
    )
    
    # Upload to presigned URL
    async with httpx.AsyncClient() as http_client:
        with open(file_path, 'rb') as f:
            await http_client.put(upload_info.presigned_url, content=f.read())
    
    # Commit upload
    commit = await client.commit_upload(
        repo_name=repo_name,
        upload_id=upload_info.upload_id,
        message="Upload file via SDK"
    )
    
    return commit
```

## TypeScript SDK

### Installation

```bash
# From npm
npm install @blacklake/sdk

# From source
npm install git+https://github.com/blacklake/blacklake.git#subdirectory=sdks/typescript
```

### Basic Usage

```typescript
import { BlackLakeClient } from '@blacklake/sdk'

async function main() {
  // Initialize client
  const client = new BlackLakeClient({
    baseUrl: 'http://localhost:8080',
    apiKey: 'your-api-key'
  })
  
  // List repositories
  const repos = await client.listRepositories()
  console.log(`Found ${repos.length} repositories`)
  
  // Search files
  const results = await client.search('sales data', { limit: 10 })
  console.log(`Found ${results.data.total} results`)
  
  // Get repository tree
  const tree = await client.getRepositoryTree('my-repo', 'main')
  tree.data.forEach(entry => {
    console.log(`${entry.type}: ${entry.path}`)
  })
}

main().catch(console.error)
```

### React Integration

```typescript
import React, { useEffect, useState } from 'react'
import { BlackLakeClient, Repository } from '@blacklake/sdk'

function RepositoryList() {
  const [repos, setRepos] = useState<Repository[]>([])
  const [loading, setLoading] = useState(true)
  
  useEffect(() => {
    const client = new BlackLakeClient({
      baseUrl: process.env.REACT_APP_API_URL,
      apiKey: process.env.REACT_APP_API_KEY
    })
    
    client.listRepositories()
      .then(setRepos)
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [])
  
  if (loading) return <div>Loading...</div>
  
  return (
    <div>
      {repos.map(repo => (
        <div key={repo.id}>
          <h3>{repo.name}</h3>
          <p>{repo.description}</p>
        </div>
      ))}
    </div>
  )
}
```

### Node.js Usage

```typescript
import { BlackLakeClient } from '@blacklake/sdk'

const client = new BlackLakeClient({
  baseUrl: 'http://localhost:8080',
  apiKey: process.env.BLACKLAKE_API_KEY
})

// Search and process results
async function searchAndProcess(query: string) {
  const results = await client.search(query, { limit: 100 })
  
  for (const result of results.data.results) {
    console.log(`Processing: ${result.repo_name}/${result.path}`)
    
    // Get file metadata
    const metadata = await client.getFileMetadata(
      result.repo_name,
      'main',
      result.path
    )
    
    console.log('Metadata:', metadata)
  }
}
```

## Common Patterns

### Error Handling

```python
# Python
from blacklake import BlackLakeError, AuthenticationError, NotFoundError

try:
    repo = await client.get_repository("nonexistent")
except NotFoundError:
    print("Repository not found")
except AuthenticationError:
    print("Authentication failed")
except BlackLakeError as e:
    print(f"API error: {e.message}")
```

```typescript
// TypeScript
import { BlackLakeError, AuthenticationError, NotFoundError } from '@blacklake/sdk'

try {
  const repo = await client.getRepository('nonexistent')
} catch (error) {
  if (error instanceof NotFoundError) {
    console.log('Repository not found')
  } else if (error instanceof AuthenticationError) {
    console.log('Authentication failed')
  } else if (error instanceof BlackLakeError) {
    console.log(`API error: ${error.message}`)
  }
}
```

### Pagination

```python
# Python
async def get_all_repos(client):
    all_repos = []
    offset = 0
    limit = 50
    
    while True:
        results = await client.search("", limit=limit, offset=offset)
        all_repos.extend(results.results)
        
        if len(results.results) < limit:
            break
            
        offset += limit
    
    return all_repos
```

```typescript
// TypeScript
async function getAllRepos(client: BlackLakeClient) {
  const allRepos = []
  let offset = 0
  const limit = 50
  
  while (true) {
    const results = await client.search('', { limit, offset })
    allRepos.push(...results.data.results)
    
    if (results.data.results.length < limit) {
      break
    }
    
    offset += limit
  }
  
  return allRepos
}
```

### Batch Operations

```python
# Python
async def batch_upload(client, repo_name, files):
    uploads = []
    
    # Initiate all uploads
    for file_path in files:
        upload_info = await client.initiate_upload(
            repo_name=repo_name,
            path=file_path,
            size=os.path.getsize(file_path),
            content_type=mimetypes.guess_type(file_path)[0]
        )
        uploads.append(upload_info)
    
    # Upload files in parallel
    async with httpx.AsyncClient() as http_client:
        tasks = []
        for upload_info in uploads:
            task = upload_file(http_client, upload_info)
            tasks.append(task)
        
        await asyncio.gather(*tasks)
    
    # Commit all uploads
    commit = await client.commit_upload(
        repo_name=repo_name,
        upload_id=uploads[0].upload_id,  # Use first upload ID
        message=f"Batch upload of {len(files)} files"
    )
    
    return commit
```

## Examples

### Jupyter Notebook

```python
# Jupyter notebook example
import pandas as pd
from blacklake import BlackLakeClient

# Initialize client
client = BlackLakeClient(base_url="http://localhost:8080")

# Search for CSV files
results = await client.search("file_type:csv", limit=100)

# Download and analyze data
for result in results.results:
    if result.content_type == "text/csv":
        # Get file content (implement download method)
        df = pd.read_csv(f"data/{result.path}")
        print(f"Analyzing {result.path}: {df.shape}")
```

### Data Pipeline

```typescript
// TypeScript data pipeline
import { BlackLakeClient } from '@blacklake/sdk'

class DataPipeline {
  private client: BlackLakeClient
  
  constructor(apiKey: string) {
    this.client = new BlackLakeClient({ apiKey })
  }
  
  async processDataset(repoName: string, datasetPath: string) {
    // Search for related files
    const results = await this.client.search(`repo:${repoName} path:${datasetPath}`)
    
    // Process each file
    for (const result of results.data.results) {
      await this.processFile(result)
    }
  }
  
  private async processFile(result: SearchResult) {
    // Get metadata
    const metadata = await this.client.getFileMetadata(
      result.repo_name,
      'main',
      result.path
    )
    
    // Update metadata
    await this.client.updateFileMetadata(
      result.repo_name,
      'main',
      result.path,
      {
        ...metadata,
        processed: true,
        processed_at: new Date().toISOString()
      }
    )
  }
}
```

## Development

### Building from Source

```bash
# Python SDK
cd sdks/python
pip install -e .

# TypeScript SDK
cd sdks/typescript
npm install
npm run build
```

### Testing

```bash
# Python SDK
cd sdks/python
pytest

# TypeScript SDK
cd sdks/typescript
npm test
```

### Publishing

```bash
# Python SDK
cd sdks/python
python -m build
twine upload dist/*

# TypeScript SDK
cd sdks/typescript
npm publish
```
