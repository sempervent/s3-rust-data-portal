use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;
use sophia::api::graph::Graph;
use sophia::api::serializer::Stringifier;
// use sophia::turtle::TurtleSerializer; // Commented out due to import issues
use sophia::api::term::Term;
use url::Url;

// Re-export common types
pub use uuid::Uuid as RepoId;

// Custom JsonSchema implementation for UUID via wrapper
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UuidWrapper(#[schemars(with = "String")] pub Uuid);

impl From<Uuid> for UuidWrapper {
    fn from(uuid: Uuid) -> Self {
        UuidWrapper(uuid)
    }
}

impl From<UuidWrapper> for Uuid {
    fn from(wrapper: UuidWrapper) -> Self {
        wrapper.0
    }
}

// SQLx trait implementations for UuidWrapper
impl sqlx::Type<sqlx::Postgres> for UuidWrapper {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        <Uuid as sqlx::Type<sqlx::Postgres>>::type_info()
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for UuidWrapper {
    fn encode_by_ref(&self, buf: &mut <sqlx::Postgres as sqlx::database::HasArguments<'_>>::ArgumentBuffer) -> sqlx::encode::IsNull {
        self.0.encode_by_ref(buf)
    }
}

impl sqlx::Decode<'_, sqlx::Postgres> for UuidWrapper {
    fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let uuid = <Uuid as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(UuidWrapper(uuid))
    }
}
pub use uuid::Uuid as CommitId;

// Re-export validation functions
pub use validation::*;

// Re-export merge and schema functions
pub use merge::*;
pub use schema::*;

/// Repository information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Repository {
    pub id: UuidWrapper,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

/// Reference (branch or tag)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Reference {
    pub repo_id: UuidWrapper,
    pub name: String,
    pub kind: ReferenceKind,
    pub commit_id: UuidWrapper,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReferenceKind {
    Branch,
    Tag,
}

/// Commit information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Commit {
    pub id: UuidWrapper,
    pub repo_id: UuidWrapper,
    pub parent_id: Option<UuidWrapper>,
    pub author: String,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub stats: Option<serde_json::Value>,
}

/// Object metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Object {
    pub sha256: String,
    pub size: i64,
    pub media_type: Option<String>,
    pub s3_key: String,
    pub created_at: DateTime<Utc>,
}

/// Tree entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Entry {
    pub id: UuidWrapper,
    pub commit_id: UuidWrapper,
    pub path: String,
    pub object_sha256: Option<String>,
    pub meta: serde_json::Value,
    pub is_dir: bool,
    pub created_at: DateTime<Utc>,
}

/// ACL entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Acl {
    pub repo_id: UuidWrapper,
    pub subject: String,
    pub perm: Permission,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Permission {
    Read,
    Write,
    Admin,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditLog {
    pub id: i64,
    pub at: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub repo_name: Option<String>,
    pub ref_name: Option<String>,
    pub path: Option<String>,
    pub request_meta: Option<serde_json::Value>,
    pub response_meta: Option<serde_json::Value>,
}

// API Request/Response types

/// Request to initialize an upload
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UploadInitRequest {
    pub path: String,
    pub size: u64,
    pub media_type: Option<String>,
}

/// Response for upload initialization
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UploadInitResponse {
    pub upload_url: String,
    pub sha256: String,
    pub s3_key: String,
    pub expires_at: DateTime<Utc>,
}

/// Request to create a commit
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CommitRequest {
    pub r#ref: String,
    pub message: Option<String>,
    pub changes: Vec<Change>,
    pub expected_parent: Option<UuidWrapper>,
}

/// A change in a commit
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Change {
    pub op: ChangeOp,
    pub path: String,
    pub sha256: Option<String>,
    pub meta: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ChangeOp {
    Add,
    Modify,
    Delete,
    Meta,
}

/// Response for commit creation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CommitResponse {
    pub commit_id: UuidWrapper,
    pub parent_id: Option<UuidWrapper>,
    pub created_at: DateTime<Utc>,
}

/// Request to create a repository
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRepoRequest {
    pub name: String,
}

/// Response for repository creation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateRepoResponse {
    pub id: UuidWrapper,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

/// Tree listing response
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TreeResponse {
    pub entries: Vec<TreeEntry>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TreeEntry {
    pub path: String,
    pub is_dir: bool,
    pub size: Option<i64>,
    pub media_type: Option<String>,
    pub meta: serde_json::Value,
}

/// Search request
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchRequest {
    pub filters: HashMap<String, serde_json::Value>,
    pub sort: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// Search response
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchResponse {
    pub entries: Vec<SearchEntry>,
    pub total: u32,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchEntry {
    pub path: String,
    pub commit_id: UuidWrapper,
    pub meta: serde_json::Value,
    pub size: Option<i64>,
    pub media_type: Option<String>,
}

// Authentication types

/// JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub iat: u64,
    pub roles: Option<Vec<String>>,
}

/// Authentication context
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub sub: String,
    pub roles: Vec<String>,
}

// JSON Schema for metadata validation
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct MetadataSchema {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub r#type: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub required: Vec<String>,
}

impl Default for MetadataSchema {
    fn default() -> Self {
        let mut properties = HashMap::new();
        properties.insert("name".to_string(), serde_json::json!({
            "type": "string",
            "description": "Human-readable name of the artifact"
        }));
        properties.insert("description".to_string(), serde_json::json!({
            "type": "string",
            "description": "Description of the artifact"
        }));
        properties.insert("version".to_string(), serde_json::json!({
            "type": "string",
            "description": "Version of the artifact"
        }));
        properties.insert("tags".to_string(), serde_json::json!({
            "type": "array",
            "items": {"type": "string"},
            "description": "Tags for categorization"
        }));

        Self {
            schema: "http://json-schema.org/draft-07/schema#".to_string(),
            r#type: "object".to_string(),
            properties,
            required: vec!["name".to_string()],
        }
    }
}

/// Hash a file and return SHA256 as hex string
pub fn hash_file(path: &std::path::Path) -> anyhow::Result<String> {
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Hash bytes and return SHA256 as hex string
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"hello world").unwrap();
        
        let hash = hash_file(temp_file.path()).unwrap();
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_hash_bytes() {
        let hash = hash_bytes(b"hello world");
        assert_eq!(hash, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn test_metadata_schema_default() {
        let schema = MetadataSchema::default();
        assert!(schema.properties.contains_key("name"));
        assert!(schema.properties.contains_key("description"));
        assert!(schema.required.contains(&"name".to_string()));
    }
}

// Dublin Core Metadata Support

/// Dublin Core JSON-LD context
pub fn dc_context() -> serde_json::Value {
    serde_json::json!({
        "@context": {
            "dc": "http://purl.org/dc/elements/1.1/",
            "dcterms": "http://purl.org/dc/terms/",
            "xsd": "http://www.w3.org/2001/XMLSchema#"
        }
    })
}

/// Canonical metadata structure following Dublin Core mapping
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CanonicalMeta {
    // Required fields
    pub creation_dt: DateTime<Utc>,
    pub creator: String,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub org_lab: String,
    pub description: String,
    pub data_source: String,
    pub data_collection_method: String,
    pub version: String,
    
    // Optional fields
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub license: Option<String>,
}

/// Entry metadata index for fast querying
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntryMetaIndex {
    pub commit_id: UuidWrapper,
    pub path: String,
    pub creation_dt: Option<DateTime<Utc>>,
    pub creator: Option<String>,
    pub file_name: Option<String>,
    pub file_type: Option<String>,
    pub file_size: Option<i64>,
    pub org_lab: Option<String>,
    pub description: Option<String>,
    pub data_source: Option<String>,
    pub data_collection_method: Option<String>,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub tags: Option<Vec<String>>,
    pub license: Option<String>,
}

/// RDF artifact storage
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ArtifactRdf {
    pub commit_id: UuidWrapper,
    pub path: String,
    pub format: RdfFormat,
    pub graph: String,
    pub graph_sha256: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RdfFormat {
    Turtle,
    Jsonld,
}

/// Convert canonical metadata to Dublin Core JSON-LD
pub fn canonical_to_dc_jsonld(subject_iri: &str, meta: &CanonicalMeta) -> serde_json::Value {
    let mut doc = serde_json::Map::new();
    
    // Add context
    doc.insert("@context".to_string(), dc_context()["@context"].clone());
    doc.insert("@id".to_string(), serde_json::Value::String(subject_iri.to_string()));
    
    // Map canonical fields to Dublin Core
    doc.insert("dc:title".to_string(), serde_json::Value::String(meta.file_name.clone()));
    doc.insert("dc:creator".to_string(), serde_json::Value::String(meta.creator.clone()));
    doc.insert("dc:description".to_string(), serde_json::Value::String(meta.description.clone()));
    doc.insert("dcterms:created".to_string(), serde_json::Value::String(meta.creation_dt.to_rfc3339()));
    doc.insert("dc:format".to_string(), serde_json::Value::String(meta.file_type.clone()));
    doc.insert("dcterms:extent".to_string(), serde_json::Value::Number(serde_json::Number::from(meta.file_size)));
    doc.insert("dc:source".to_string(), serde_json::Value::String(meta.data_source.clone()));
    doc.insert("dcterms:methodOfAccrual".to_string(), serde_json::Value::String(meta.data_collection_method.clone()));
    doc.insert("dcterms:publisher".to_string(), serde_json::Value::String(meta.org_lab.clone()));
    doc.insert("dcterms:hasVersion".to_string(), serde_json::Value::String(meta.version.clone()));
    
    // Optional fields
    if let Some(license) = &meta.license {
        doc.insert("dcterms:license".to_string(), serde_json::Value::String(license.clone()));
    }
    
    if let Some(tags) = &meta.tags {
        let subjects: Vec<serde_json::Value> = tags.iter()
            .map(|tag| serde_json::Value::String(tag.clone()))
            .collect();
        doc.insert("dc:subject".to_string(), serde_json::Value::Array(subjects));
    }
    
    serde_json::Value::Object(doc)
}

/// Convert Dublin Core JSON-LD to Turtle format
pub fn dc_jsonld_to_turtle(doc: &serde_json::Value) -> anyhow::Result<String> {
    // For now, implement a simple Turtle serializer
    // In a full implementation, you'd use sophia's JSON-LD parser and Turtle serializer
    let mut turtle = String::new();
    
    if let Some(id) = doc.get("@id") {
        if let Some(subject) = id.as_str() {
            turtle.push_str(&format!("<{}> ", subject));
            turtle.push_str("a <http://purl.org/dc/dcmitype/Dataset> ;\n");
            
            // Add properties
            for (key, value) in doc.as_object().unwrap() {
                if key.starts_with("@") {
                    continue;
                }
                
                let predicate = match key.as_str() {
                    "dc:title" => "<http://purl.org/dc/elements/1.1/title>",
                    "dc:creator" => "<http://purl.org/dc/elements/1.1/creator>",
                    "dc:description" => "<http://purl.org/dc/elements/1.1/description>",
                    "dcterms:created" => "<http://purl.org/dc/terms/created>",
                    "dc:format" => "<http://purl.org/dc/elements/1.1/format>",
                    "dcterms:extent" => "<http://purl.org/dc/terms/extent>",
                    "dc:source" => "<http://purl.org/dc/elements/1.1/source>",
                    "dcterms:methodOfAccrual" => "<http://purl.org/dc/terms/methodOfAccrual>",
                    "dcterms:publisher" => "<http://purl.org/dc/terms/publisher>",
                    "dcterms:hasVersion" => "<http://purl.org/dc/terms/hasVersion>",
                    "dcterms:license" => "<http://purl.org/dc/terms/license>",
                    "dc:subject" => "<http://purl.org/dc/elements/1.1/subject>",
                    _ => continue,
                };
                
                match value {
                    serde_json::Value::String(s) => {
                        turtle.push_str(&format!("    {} \"{}\" ;\n", predicate, s));
                    }
                    serde_json::Value::Number(n) => {
                        turtle.push_str(&format!("    {} {} ;\n", predicate, n));
                    }
                    serde_json::Value::Array(arr) => {
                        for item in arr {
                            if let Some(s) = item.as_str() {
                                turtle.push_str(&format!("    {} \"{}\" ;\n", predicate, s));
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // Remove trailing semicolon and add period
            turtle = turtle.trim_end_matches(" ;\n").to_string();
            turtle.push_str(" .\n");
        }
    }
    
    Ok(turtle)
}

/// Convert canonical metadata directly to Turtle
pub fn canonical_to_turtle(subject_iri: &str, meta: &CanonicalMeta) -> anyhow::Result<String> {
    let jsonld = canonical_to_dc_jsonld(subject_iri, meta);
    dc_jsonld_to_turtle(&jsonld)
}

/// Project JSONB metadata to entry_meta_index row
pub fn project_to_index(commit_id: Uuid, path: &str, meta: &serde_json::Value) -> EntryMetaIndex {
    EntryMetaIndex {
        commit_id: UuidWrapper(commit_id),
        path: path.to_string(),
        creation_dt: meta.get("creation_dt")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        creator: meta.get("creator").and_then(|v| v.as_str()).map(|s| s.to_string()),
        file_name: meta.get("file_name").and_then(|v| v.as_str()).map(|s| s.to_string()),
        file_type: meta.get("file_type").and_then(|v| v.as_str()).map(|s| s.to_string()),
        file_size: meta.get("file_size").and_then(|v| v.as_i64()),
        org_lab: meta.get("org_lab").and_then(|v| v.as_str()).map(|s| s.to_string()),
        description: meta.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        data_source: meta.get("data_source").and_then(|v| v.as_str()).map(|s| s.to_string()),
        data_collection_method: meta.get("data_collection_method").and_then(|v| v.as_str()).map(|s| s.to_string()),
        version: meta.get("version").and_then(|v| v.as_str()).map(|s| s.to_string()),
        notes: meta.get("notes").and_then(|v| v.as_str()).map(|s| s.to_string()),
        tags: meta.get("tags").and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect()),
        license: meta.get("license").and_then(|v| v.as_str()).map(|s| s.to_string()),
    }
}

/// Generate subject IRI for an artifact
pub fn generate_subject_iri(repo: &str, r#ref: &str, path: &str) -> String {
    let encoded_path = urlencoding::encode(path);
    format!("https://blacklake.local/{}/{}/{}", repo, r#ref, encoded_path)
}

#[cfg(test)]
mod metadata_tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_canonical_to_dc_jsonld() {
        let meta = CanonicalMeta {
            creation_dt: Utc.with_ymd_and_hms(2025, 1, 17, 18, 28, 0).unwrap(),
            creator: "you@example.org".to_string(),
            file_name: "demo.csv".to_string(),
            file_type: "text/csv".to_string(),
            file_size: 1234,
            org_lab: "ORNL".to_string(),
            description: "Demo dataset".to_string(),
            data_source: "sensor".to_string(),
            data_collection_method: "manual".to_string(),
            version: "1.0".to_string(),
            notes: Some("Test notes".to_string()),
            tags: Some(vec!["demo".to_string(), "csv".to_string()]),
            license: Some("CC-BY-4.0".to_string()),
        };

        let subject_iri = "https://blacklake.local/mylab/main/datasets/demo.csv";
        let jsonld = canonical_to_dc_jsonld(subject_iri, &meta);

        assert!(jsonld.get("@context").is_some());
        assert_eq!(jsonld.get("@id").unwrap().as_str().unwrap(), subject_iri);
        assert_eq!(jsonld.get("dc:title").unwrap().as_str().unwrap(), "demo.csv");
        assert_eq!(jsonld.get("dc:creator").unwrap().as_str().unwrap(), "you@example.org");
    }

    #[test]
    fn test_dc_jsonld_to_turtle() {
        let jsonld = serde_json::json!({
            "@context": {
                "dc": "http://purl.org/dc/elements/1.1/",
                "dcterms": "http://purl.org/dc/terms/"
            },
            "@id": "https://example.org/test",
            "dc:title": "Test Dataset",
            "dc:creator": "Test Creator"
        });

        let turtle = dc_jsonld_to_turtle(&jsonld).unwrap();
        assert!(turtle.contains("<https://example.org/test>"));
        assert!(turtle.contains("dc:title"));
        assert!(turtle.contains("Test Dataset"));
    }

    #[test]
    fn test_project_to_index() {
        let meta = serde_json::json!({
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
            "tags": ["demo", "csv"]
        });

        let commit_id = Uuid::new_v4();
        let index = project_to_index(commit_id, "datasets/demo.csv", &meta);

        assert_eq!(index.commit_id, commit_id);
        assert_eq!(index.path, "datasets/demo.csv");
        assert_eq!(index.file_name, Some("demo.csv".to_string()));
        assert_eq!(index.file_size, Some(1234));
        assert_eq!(index.tags, Some(vec!["demo".to_string(), "csv".to_string()]));
    }
}

// Module declarations
pub mod validation;
pub mod merge;
pub mod schema;
pub mod governance;
pub mod jobs;
pub mod policy;
pub mod search;
pub mod sessions;
pub mod embeddings;
pub mod compliance;
pub mod compliance_jobs;
pub mod compliance_worker;
pub mod observability;

#[cfg(test)]
mod governance_tests;
#[cfg(test)]
mod jobs_tests;
#[cfg(test)]
mod policy_tests;
#[cfg(test)]
mod sessions_tests;
#[cfg(test)]
mod search_tests;
