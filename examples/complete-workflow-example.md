# Complete BlackLake CLI Workflow Example

This example demonstrates the complete workflow from initializing artifacts with metadata to uploading them to BlackLake.

## Step 1: Initialize Artifacts with Metadata

### Initialize a Single File
```bash
# Create a sample data file
echo "name,age,city
John,30,New York
Jane,25,Los Angeles" > sample-data.csv

# Initialize with BlackLake metadata
blacklake-cli init sample-data.csv \
  --set "provenance.author.name=Data Scientist" \
  --set "provenance.author.email=datascientist@example.com" \
  --set "provenance.organization=Research Lab" \
  --set "authorization.access_level=internal" \
  --set "authorization.data_classification=internal"
```

This creates `sample-data.csv.bl.metadata.yaml` with comprehensive metadata.

### Initialize a Directory
```bash
# Create a dataset directory
mkdir my-dataset
echo "Sample data" > my-dataset/data.csv
echo "Configuration" > my-dataset/config.json
echo "README" > my-dataset/README.txt

# Initialize the directory
blacklake-cli init my-dataset \
  --set "provenance.author.name=Research Team" \
  --set "provenance.organization=University Lab" \
  --set "authorization.access_level=restricted"
```

This creates:
- `my-dataset/.bl/data.csv.metadata.yaml`
- `my-dataset/.bl/config.json.metadata.yaml`
- `my-dataset/.bl/README.txt.metadata.yaml`
- `my-dataset/.bl/directory.metadata.yaml`

## Step 2: Upload with Metadata

### Upload a Single File
```bash
# Upload the file with its metadata
blacklake-cli put my-repo main sample-data.csv data/sample-data.csv
```

The CLI automatically detects `sample-data.csv.bl.metadata.yaml` and uses it for the upload.

### Upload a Directory
```bash
# Upload the entire directory with metadata
blacklake-cli put my-repo main my-dataset datasets/my-dataset
```

The CLI automatically:
1. Detects the `.bl/` directory
2. Uses individual file metadata where available
3. Falls back to directory metadata for files without specific metadata
4. Uploads all files in a single commit

## Generated Metadata Structure

The `init` command creates comprehensive metadata templates:

### Artifact Information
```yaml
artifact:
  name: "sample-data.csv"
  description: "BlackLake artifact: sample-data.csv"
  version: "1.0.0"
  tags: ["data", "tabular"]
  file_type: "tabular"
  mime_type: "text/csv"
```

### Authorization (Security-First Defaults)
```yaml
authorization:
  access_level: "restricted"  # Most restrictive by default
  data_classification: "confidential"  # Most restrictive by default
  retention_policy: "indefinite"
  legal_hold: false
  export_restrictions: ["none"]
  access_controls:
    read_users: []      # No access by default
    read_groups: []
    write_users: []
    write_groups: []
    admin_users: []
    admin_groups: []
```

### Provenance
```yaml
provenance:
  author:
    name: "Data Scientist"
    email: "datascientist@example.com"
    affiliation: "Research Lab"
  organization: "Research Lab"
  project: "Unknown Project"
  license: "Unknown License"
```

### Technical Information
```yaml
technical:
  created_at: "2024-09-25T16:30:00Z"
  modified_at: "2024-09-25T16:30:00Z"
  file_size: 1024
  checksum_sha256: ""
  format_version: "1.0"
  environment:
    os: "macos"
    software_versions: {}
```

## Metadata Conversion

When uploading, the CLI converts BlackLake metadata to the canonical format:

### BlackLake Metadata → Canonical Metadata
- **Artifact name** → `file_name`
- **Author email** → `creator`
- **Organization** → `org_lab`
- **Description** → `description`
- **Tags** → `tags`
- **License** → `license`
- **Authorization info** → `notes` (for audit trail)
- **Custom fields** → `notes` (preserved)

## Security and Compliance

The workflow ensures:

1. **Security by Default**: All artifacts start with the most restrictive access settings
2. **Audit Trail**: Authorization and provenance information is preserved
3. **Compliance**: Legal hold, retention policies, and data classification are tracked
4. **Traceability**: Full provenance from creation to upload

## Example Output

### Initialize Output
```
🚀 Initializing BlackLake artifact: sample-data.csv
📄 Created metadata: sample-data.csv.bl.metadata.yaml
✅ BlackLake artifact initialized successfully!
```

### Upload Output
```
🚀 Uploading sample-data.csv to my-repo/data/sample-data.csv
📋 Found BlackLake metadata: sample-data.csv.bl.metadata.yaml
📤 Uploading file...
💾 Committing changes...
✅ Successfully committed: UuidWrapper(12345678-1234-1234-1234-123456789abc)
```

### Directory Upload Output
```
🚀 Uploading my-dataset to my-repo/datasets/my-dataset
📁 Uploading directory with metadata...
  📄 Uploaded: my-dataset/data.csv
  📄 Uploaded: my-dataset/config.json
  📄 Uploaded: my-dataset/README.txt
💾 Committing 3 files...
✅ Successfully committed directory: UuidWrapper(87654321-4321-4321-4321-cba987654321)
```

## Benefits

1. **Automated Metadata**: No need to manually enter metadata during upload
2. **Consistent Structure**: Standardized metadata across all artifacts
3. **Security First**: Defaults to most restrictive settings
4. **Audit Compliance**: Full provenance and authorization tracking
5. **Batch Operations**: Upload entire directories with individual file metadata
6. **Flexible Configuration**: Override defaults with command-line options

This workflow provides a complete solution for managing data artifacts with comprehensive metadata, security controls, and compliance tracking.

