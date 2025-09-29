# BlackLake CLI Init Command Examples

The `blacklake-cli init` command allows you to initialize directories or files as BlackLake artifacts with comprehensive metadata templates.

## Basic Usage

### Initialize a File
```bash
# Initialize a single file
blacklake-cli init data.csv

# This creates: data.csv.bl.metadata.yaml
```

### Initialize a Directory
```bash
# Initialize a directory
blacklake-cli init ./my-dataset/

# This creates: ./my-dataset/.bl/ directory with metadata for each file
# - ./my-dataset/.bl/data.csv.metadata.yaml
# - ./my-dataset/.bl/README.txt.metadata.yaml
# - ./my-dataset/.bl/directory.metadata.yaml
```

## Advanced Usage

### Set Custom Metadata
```bash
# Set metadata using dot notation
blacklake-cli init data.csv \
  --set "provenance.author.name=John Doe" \
  --set "provenance.author.email=john@example.com" \
  --set "provenance.organization=My Lab" \
  --set "authorization.access_level=public" \
  --set "authorization.data_classification=unclassified"
```

### Force Overwrite
```bash
# Overwrite existing metadata files
blacklake-cli init data.csv --force
```

### Dry Run
```bash
# See what would be created without actually creating files
blacklake-cli init data.csv --dry-run
```

## Generated Metadata Structure

The init command creates comprehensive metadata templates with the following structure:

### Artifact Information
- **name**: File/directory name
- **description**: Auto-generated description
- **version**: Default "1.0.0"
- **tags**: Auto-detected based on file type
- **file_type**: Categorized file type (text, tabular, structured, etc.)
- **mime_type**: Detected MIME type

### Authorization (Default: Most Restrictive)
- **access_level**: "restricted" (public, internal, restricted, confidential)
- **data_classification**: "confidential" (unclassified, internal, confidential, secret)
- **retention_policy**: "indefinite"
- **legal_hold**: false
- **export_restrictions**: ["none"]
- **access_controls**: Empty arrays for users/groups

### Provenance
- **author**: Default "Unknown Author"
- **organization**: Default "Unknown Organization"
- **project**: Default "Unknown Project"
- **license**: Default "Unknown License"

### Technical Information
- **created_at**: Current timestamp
- **modified_at**: Current timestamp
- **file_size**: Actual file size
- **checksum_sha256**: Empty (to be calculated later)
- **environment**: OS and software version info

### Content Information
- **language**: "en"
- **encoding**: "utf-8"
- **schema**: None
- **validation_rules**: Empty array
- **quality_metrics**: Empty object

## File Type Detection

The init command automatically detects and categorizes files:

| Extension | File Type | MIME Type |
|-----------|-----------|-----------|
| .txt, .md | text | text/plain |
| .csv, .tsv | tabular | text/csv |
| .json, .yaml | structured | application/json |
| .pdf | document | application/pdf |
| .png, .jpg | image | image/png |
| .mp4 | video | video/mp4 |
| .mp3 | audio | audio/mpeg |
| .py, .r | code | text/plain |

## Security Defaults

The init command defaults to the most restrictive security settings:

- **Access Level**: `restricted` (not public)
- **Data Classification**: `confidential` (not unclassified)
- **Legal Hold**: `false` (can be set to true if needed)
- **Access Controls**: Empty (no users/groups have access by default)

This ensures that sensitive data is protected by default and requires explicit configuration to make it accessible.

## Example Generated Metadata

```yaml
artifact:
  name: "data.csv"
  description: "BlackLake artifact: data.csv"
  version: "1.0.0"
  tags: ["data", "tabular"]
  file_type: "tabular"
  mime_type: "text/csv"

authorization:
  access_level: "restricted"
  data_classification: "confidential"
  retention_policy: "indefinite"
  legal_hold: false
  export_restrictions: ["none"]
  access_controls:
    read_users: []
    read_groups: []
    write_users: []
    write_groups: []
    admin_users: []
    admin_groups: []

provenance:
  author:
    name: "Unknown Author"
    email: "unknown@example.com"
    orcid: null
    affiliation: "Unknown Organization"
  organization: "Unknown Organization"
  project: "Unknown Project"
  funding: null
  license: "Unknown License"
  citation: null
  related_artifacts: []

technical:
  created_at: "2024-09-25T16:30:00Z"
  modified_at: "2024-09-25T16:30:00Z"
  file_size: 1024
  checksum_sha256: ""
  format_version: "1.0"
  dependencies: []
  environment:
    os: "macos"
    software_versions: {}
    hardware: null
    cloud_provider: null

content:
  language: "en"
  encoding: "utf-8"
  schema: null
  validation_rules: []
  quality_metrics: {}

custom: {}
```

## Integration with BlackLake

After initialization, you can:

1. **Edit metadata**: Use `blacklake-cli meta edit` to modify the generated metadata
2. **Upload artifacts**: Use `blacklake-cli put` to upload the files with their metadata
3. **Search artifacts**: Use `blacklake-cli search` to find artifacts by metadata
4. **Manage access**: Update authorization settings as needed

The init command provides a solid foundation for BlackLake artifact management with security-first defaults.

