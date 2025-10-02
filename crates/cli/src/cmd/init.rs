use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use serde::{Deserialize, Serialize};
use serde_yaml;
use time::OffsetDateTime;
use walkdir::WalkDir;
use blake3::Hasher as Blake3Hasher;
use sha2::{Sha256, Digest};
use mime_guess;
use regex::Regex;
use thiserror::Error;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct InitArgs {
    pub path: String,
    pub recursive: bool,
    pub max_depth: u32,
    pub include_hidden: bool,
    pub follow_symlinks: bool,
    pub namespace: String,
    pub label: Vec<(String, String)>,
    pub meta: Vec<(String, String)>,
    pub class: String,
    pub owner: Option<String>,
    pub no_hash: bool,
    pub hash: String,
    pub set: Vec<(String, String)>,
    pub overwrite: bool,
    pub dry_run: bool,
    pub with_authorization: bool,
}

#[derive(Error, Debug)]
pub enum InitError {
    #[error("Path does not exist: {0}")]
    PathNotFound(String),
    #[error("Path is neither file nor directory: {0}")]
    InvalidPath(String),
    #[error("Metadata file already exists: {0}")]
    MetadataExists(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Invalid dot notation: {0}")]
    InvalidDotNotation(String),
}

// Per-file metadata template
#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub schema_version: String,
    pub artifact_type: String,
    pub name: String,
    pub namespace: String,
    pub media_type: String,
    pub size: u64,
    pub created_at: String,
    pub source_path: String,
    pub checksums: Checksums,
    pub labels: HashMap<String, String>,
    pub user_metadata: HashMap<String, String>,
    pub immutability: Immutability,
    pub policy: Policy,
    pub auth: Auth,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checksums {
    pub blake3: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Immutability {
    pub requested: bool,
    pub object_lock: bool,
    pub retention_days: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    pub classification: String,
    pub owner: String,
    pub custodians: Vec<String>,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
    pub shareable: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Auth {
    pub required: bool,
    pub min_auth: String,
    pub allowed_audiences: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Provenance {
    pub parents: Vec<String>,
    pub generated_by: GeneratedBy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneratedBy {
    pub tool: String,
    pub version: String,
    pub command: String,
}

// Directory-level manifest
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryMetadata {
    pub schema_version: String,
    pub artifact_type: String,
    pub name: String,
    pub namespace: String,
    pub created_at: String,
    pub labels: HashMap<String, String>,
    pub user_metadata: HashMap<String, String>,
    pub immutability: Immutability,
    pub policy: Policy,
    pub auth: Auth,
    pub manifest: Manifest,
    pub provenance: Provenance,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub media_type: String,
    pub checksums: Checksums,
}

// Strict policy template
#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyTemplate {
    pub schema_version: String,
    pub default: PolicyDefault,
    pub bindings: Vec<PolicyBinding>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyDefault {
    pub classification: String,
    pub immutable: bool,
    pub shareable: bool,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyBinding {
    pub r#match: PolicyMatch,
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PolicyMatch {
    pub path_glob: String,
}

// Authorization template
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationTemplate {
    pub schema_version: String,
    pub license: String,
    pub permitted_uses: Vec<String>,
    pub obligations: Vec<String>,
    pub prohibited_uses: Vec<String>,
    pub notes: String,
}

// Provenance stub
#[derive(Debug, Serialize, Deserialize)]
pub struct ProvenanceStub {
    pub schema_version: String,
    pub lineage: Lineage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Lineage {
    pub parents: Vec<String>,
    pub notes: String,
}

pub async fn init_command(args: InitArgs) -> Result<()> {
    let path = Path::new(&args.path);
    
    if !path.exists() {
        return Err(InitError::PathNotFound(args.path.clone()).into());
    }

    let is_directory = path.is_dir();
    let is_file = path.is_file();

    if !is_directory && !is_file {
        return Err(InitError::InvalidPath(args.path.clone()).into());
    }

    println!("🚀 Initializing BlackLake artifact: {}", args.path);
    
    if args.dry_run {
        println!("🔍 Dry run mode - no files will be created");
    }

    if is_directory {
        init_directory(path, &args).await?;
    } else {
        init_file(path, &args).await?;
    }

    println!("✅ BlackLake artifact initialized successfully!");
    Ok(())
}

async fn init_directory(dir_path: &Path, args: &InitArgs) -> Result<()> {
    let bl_dir = dir_path.join(".bl");
    
    if bl_dir.exists() && !args.overwrite {
        return Err(InitError::MetadataExists(bl_dir.display().to_string()).into());
    }

    if !args.dry_run {
        fs::create_dir_all(&bl_dir)?;
        
        // Create Git-like structure
        fs::create_dir_all(&bl_dir.join("hooks"))?;
        fs::create_dir_all(&bl_dir.join("refs"))?;
        fs::create_dir_all(&bl_dir.join("refs/tags"))?;
        fs::create_dir_all(&bl_dir.join("objects"))?;
        fs::create_dir_all(&bl_dir.join("objects/blobs"))?;
        fs::create_dir_all(&bl_dir.join("objects/commits"))?;
        
        // Create configuration file
        create_config_file(&bl_dir, args)?;
        
        // Create comprehensive metadata template
        create_metadata_template(&bl_dir, args)?;
        
        // Create .gitignore
        create_gitignore(&bl_dir)?;
        
        // Create hooks
        create_hooks(&bl_dir)?;
        
        // Set restrictive permissions on Unix
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(&bl_dir)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&bl_dir, perms)?;
        }
    }

    // Collect files to process
    let mut files = Vec::new();
    let walker = WalkDir::new(dir_path)
        .max_depth(if args.recursive { args.max_depth as usize } else { 1 })
        .follow_links(args.follow_symlinks)
        .into_iter()
        .filter_entry(|e| {
            if !args.include_hidden {
                let name = e.file_name().to_string_lossy();
                if name.starts_with('.') && name != "." && name != ".." {
                    return false;
                }
            }
            true
        });

    for entry in walker {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    // Create per-file metadata
    let mut file_entries = Vec::new();
    for file_path in &files {
        let relative_path = file_path.strip_prefix(dir_path)
            .unwrap_or(file_path)
            .to_string_lossy()
            .to_string();
        
        let metadata_path = bl_dir.join(format!("{}.metadata.yaml", 
            file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")));

        if metadata_path.exists() && !args.overwrite {
            println!("⚠️  Skipping existing metadata: {}", metadata_path.display());
            continue;
        }

        let metadata = create_file_metadata(file_path, &relative_path, args)?;
        let file_entry = create_file_entry(file_path, &relative_path, args)?;
        file_entries.push(file_entry);

        if !args.dry_run {
            write_metadata_file(&metadata_path, &metadata)?;
            println!("📄 Created metadata: {}", metadata_path.display());
        } else {
            println!("📄 Would create metadata: {}", metadata_path.display());
        }
    }

    // Create directory-level manifest
    let dir_metadata_path = bl_dir.join("_artifact.metadata.yaml");
    let dir_metadata = create_directory_metadata(dir_path, &file_entries, args)?;
    
    if !args.dry_run {
        write_metadata_file(&dir_metadata_path, &dir_metadata)?;
        println!("📁 Created directory manifest: {}", dir_metadata_path.display());
    } else {
        println!("📁 Would create directory manifest: {}", dir_metadata_path.display());
    }

    // Create policy template
    let policy_path = bl_dir.join("policy.yaml");
    let policy = create_policy_template(args)?;
    
    if !args.dry_run {
        write_metadata_file(&policy_path, &policy)?;
        println!("📋 Created policy template: {}", policy_path.display());
    } else {
        println!("📋 Would create policy template: {}", policy_path.display());
    }

    // Create authorization template
    let auth_path = bl_dir.join("authorization.yaml");
    let auth = create_authorization_template(args)?;
    
    if !args.dry_run {
        write_metadata_file(&auth_path, &auth)?;
        println!("🔐 Created authorization template: {}", auth_path.display());
    } else {
        println!("🔐 Would create authorization template: {}", auth_path.display());
    }

    // Create README.md
    let readme_path = bl_dir.join("README.md");
    let readme_content = create_readme_content();
    
    if !args.dry_run {
        fs::write(&readme_path, readme_content)?;
        println!("📖 Created README: {}", readme_path.display());
    } else {
        println!("📖 Would create README: {}", readme_path.display());
    }

    // Create .gitignore
    let gitignore_path = bl_dir.join(".gitignore");
    let gitignore_content = create_gitignore_content();
    
    if !args.dry_run {
        fs::write(&gitignore_path, gitignore_content)?;
        println!("🚫 Created .gitignore: {}", gitignore_path.display());
    } else {
        println!("🚫 Would create .gitignore: {}", gitignore_path.display());
    }

    // Create provenance stub
    let provenance_path = bl_dir.join("provenance.yaml");
    let provenance = create_provenance_stub();
    
    if !args.dry_run {
        write_metadata_file(&provenance_path, &provenance)?;
        println!("🔗 Created provenance stub: {}", provenance_path.display());
    } else {
        println!("🔗 Would create provenance stub: {}", provenance_path.display());
    }

    Ok(())
}

async fn init_file(file_path: &Path, args: &InitArgs) -> Result<()> {
    let metadata_path = file_path.with_extension("bl.metadata.yaml");
    
    if metadata_path.exists() && !args.overwrite {
        return Err(InitError::MetadataExists(metadata_path.display().to_string()).into());
    }

    let metadata = create_file_metadata(file_path, "", args)?;
    
    if !args.dry_run {
        write_metadata_file(&metadata_path, &metadata)?;
        println!("📄 Created metadata: {}", metadata_path.display());
    } else {
        println!("📄 Would create metadata: {}", metadata_path.display());
    }

    // Create authorization template if requested
    if args.with_authorization {
        let auth_path = file_path.with_extension("bl.authorization.yaml");
        
        if auth_path.exists() && !args.overwrite {
            println!("⚠️  Skipping existing authorization: {}", auth_path.display());
        } else {
            let auth = create_authorization_template(args)?;
            
            if !args.dry_run {
                write_metadata_file(&auth_path, &auth)?;
                println!("🔐 Created authorization: {}", auth_path.display());
            } else {
                println!("🔐 Would create authorization: {}", auth_path.display());
            }
        }
    }

    Ok(())
}

fn create_file_metadata(file_path: &Path, relative_path: &str, args: &InitArgs) -> Result<FileMetadata> {
    let file_name = file_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    let media_type = mime_guess::from_path(file_path)
        .first_or_octet_stream()
        .to_string();
    
    let size = if file_path.is_file() {
        fs::metadata(file_path)?.len()
    } else {
        0
    };

    let checksums = if args.no_hash {
        Checksums {
            blake3: None,
            sha256: None,
        }
    } else {
        compute_checksums(file_path, &args.hash)?
    };

    let labels = args.label.iter().cloned().collect();
    let user_metadata = args.meta.iter().cloned().collect();
    let owner = args.owner.clone().unwrap_or_else(|| {
        format!("{}@{}", 
            std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            hostname::get().unwrap_or_default().to_string_lossy().to_string()
        )
    });

    let mut metadata = FileMetadata {
        schema_version: "1".to_string(),
        artifact_type: "file".to_string(),
        name: file_name,
        namespace: args.namespace.clone(),
        media_type,
        size,
        created_at: OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap(),
        source_path: relative_path.to_string(),
        checksums,
        labels,
        user_metadata,
        immutability: Immutability {
            requested: true,
            object_lock: false,
            retention_days: None,
        },
        policy: Policy {
            classification: args.class.clone(),
            owner,
            custodians: vec![],
            readers: vec![],
            writers: vec![],
            shareable: false,
        },
        auth: Auth {
            required: true,
            min_auth: "oidc".to_string(),
            allowed_audiences: vec![],
        },
        provenance: Provenance {
            parents: vec![],
            generated_by: GeneratedBy {
                tool: "blacklake-cli".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                command: "init".to_string(),
            },
        },
    };

    // Apply dot notation overrides
    apply_dot_notation(&mut metadata, &args.set)?;

    Ok(metadata)
}

fn create_file_entry(file_path: &Path, relative_path: &str, args: &InitArgs) -> Result<FileEntry> {
    let media_type = mime_guess::from_path(file_path)
        .first_or_octet_stream()
        .to_string();
    
    let size = if file_path.is_file() {
        fs::metadata(file_path)?.len()
    } else {
        0
    };

    let checksums = if args.no_hash {
        Checksums {
            blake3: None,
            sha256: None,
        }
    } else {
        compute_checksums(file_path, &args.hash)?
    };

    Ok(FileEntry {
        path: relative_path.to_string(),
        size,
        media_type,
        checksums,
    })
}

fn create_directory_metadata(dir_path: &Path, file_entries: &[FileEntry], args: &InitArgs) -> Result<DirectoryMetadata> {
    let dir_name = dir_path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let labels = args.label.iter().cloned().collect();
    let user_metadata = args.meta.iter().cloned().collect();
    let owner = args.owner.clone().unwrap_or_else(|| {
        format!("{}@{}", 
            std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()),
            hostname::get().unwrap_or_default().to_string_lossy().to_string()
        )
    });

    let mut metadata = DirectoryMetadata {
        schema_version: "1".to_string(),
        artifact_type: "directory".to_string(),
        name: dir_name,
        namespace: args.namespace.clone(),
        created_at: OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap(),
        labels,
        user_metadata,
        immutability: Immutability {
            requested: true,
            object_lock: false,
            retention_days: None,
        },
        policy: Policy {
            classification: args.class.clone(),
            owner,
            custodians: vec![],
            readers: vec![],
            writers: vec![],
            shareable: false,
        },
        auth: Auth {
            required: true,
            min_auth: "oidc".to_string(),
            allowed_audiences: vec![],
        },
        manifest: Manifest {
            files: file_entries.to_vec(),
        },
        provenance: Provenance {
            parents: vec![],
            generated_by: GeneratedBy {
                tool: "blacklake-cli".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                command: "init".to_string(),
            },
        },
    };

    // Apply dot notation overrides
    apply_dot_notation(&mut metadata, &args.set)?;

    Ok(metadata)
}

fn create_policy_template(args: &InitArgs) -> Result<PolicyTemplate> {
    Ok(PolicyTemplate {
        schema_version: "1".to_string(),
        default: PolicyDefault {
            classification: args.class.clone(),
            immutable: true,
            shareable: false,
            readers: vec![],
            writers: vec![],
        },
        bindings: vec![PolicyBinding {
            r#match: PolicyMatch {
                path_glob: "**/*".to_string(),
            },
            allow: vec![],
            deny: vec!["read".to_string(), "write".to_string(), "delete".to_string(), "share".to_string()],
        }],
    })
}

fn create_authorization_template(_args: &InitArgs) -> Result<AuthorizationTemplate> {
    Ok(AuthorizationTemplate {
        schema_version: "1".to_string(),
        license: "NONE".to_string(),
        permitted_uses: vec![],
        obligations: vec![],
        prohibited_uses: vec!["all".to_string()],
        notes: "Authorization is strict by default. Amend consciously.".to_string(),
    })
}

fn create_provenance_stub() -> ProvenanceStub {
    ProvenanceStub {
        schema_version: "1".to_string(),
        lineage: Lineage {
            parents: vec![],
            notes: "Edit when deriving new artifacts.".to_string(),
        },
    }
}

fn create_readme_content() -> &'static str {
    r#"# BlackLake Artifact Metadata

This directory contains metadata for a BlackLake artifact.

## Files

- `_artifact.metadata.yaml` - Directory-level manifest
- `*.metadata.yaml` - Per-file metadata sidecars
- `policy.yaml` - Access control policy (strict by default)
- `authorization.yaml` - Usage authorization (restrictive by default)
- `provenance.yaml` - Lineage and derivation information

## Usage

These metadata files are consumed by `blacklake put` when uploading artifacts.
The strict defaults ensure no unauthorized access - modify consciously.

## Security

- Default policy: no readers, no writers, immutable
- Default authorization: no permitted uses, all prohibited
- Modify these files to grant appropriate access
"#
}

fn create_gitignore_content() -> &'static str {
    r#"# Ignore local inspection outputs; keep metadata by default.
# Uncomment to ignore metadata if needed:
# *.metadata.yaml
selftest/
"#
}

fn compute_checksums(file_path: &Path, hash_algorithms: &str) -> Result<Checksums> {
    let mut file = fs::File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut blake3_hash = None;
    let mut sha256_hash = None;

    for algo in hash_algorithms.split(',') {
        match algo.trim() {
            "blake3" => {
                let mut hasher = Blake3Hasher::new();
                hasher.update(&buffer);
                blake3_hash = Some(hasher.finalize().to_hex().to_string());
            },
            "sha256" => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                sha256_hash = Some(format!("{:x}", hasher.finalize()));
            },
            _ => {} // Skip unknown algorithms
        }
    }

    Ok(Checksums {
        blake3: blake3_hash,
        sha256: sha256_hash,
    })
}

fn write_metadata_file<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let yaml_content = serde_yaml::to_string(data)?;
    
    // Write to temp file first, then atomic rename
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, yaml_content)?;
    fs::rename(&temp_path, path)?;
    
    // Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(path, perms)?;
    }
    
    Ok(())
}

fn apply_dot_notation<T: Serialize + for<'de> Deserialize<'de>>(
    data: &mut T, 
    overrides: &[(String, String)]
) -> Result<()> {
    let mut json_value = serde_json::to_value(&*data)?;
    for (key, value) in overrides {
        set_nested_value(&mut json_value, key, value)?;
    }
    *data = serde_json::from_value(json_value)?;
    Ok(())
}

fn set_nested_value(value: &mut serde_json::Value, path: &str, val: &str) -> Result<()> {
    let parsed_val = if val.starts_with('{') || val.starts_with('[') || 
        val.parse::<i64>().is_ok() || val.parse::<f64>().is_ok() || 
        val == "true" || val == "false" || val == "null" {
        serde_json::from_str(val)?
    } else {
        serde_json::Value::String(val.to_string())
    };

    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Handle array indices like "policy.readers[0]"
            if let Some(captures) = Regex::new(r"^(.+)\[(\d+)\]$").unwrap().captures(part) {
                let key = captures.get(1).unwrap().as_str();
                let index: usize = captures.get(2).unwrap().as_str().parse().unwrap();
                
                if !current.is_object() {
                    *current = serde_json::Value::Object(serde_json::Map::new());
                }
                
                let obj = current.as_object_mut().unwrap();
                if !obj.contains_key(key) {
                    obj.insert(key.to_string(), serde_json::Value::Array(vec![]));
                }
                
                let arr = obj.get_mut(key).unwrap().as_array_mut().unwrap();
                while arr.len() <= index {
                    arr.push(serde_json::Value::Null);
                }
                arr[index] = parsed_val.clone();
            } else {
                if !current.is_object() {
                    *current = serde_json::Value::Object(serde_json::Map::new());
                }
                current.as_object_mut().unwrap().insert(part.to_string(), parsed_val.clone());
            }
        } else {
            if !current.is_object() {
                *current = serde_json::Value::Object(serde_json::Map::new());
            }
            
            if !current.as_object().unwrap().contains_key(*part) {
                current.as_object_mut().unwrap().insert(part.to_string(), serde_json::Value::Object(serde_json::Map::new()));
            }
            current = current.as_object_mut().unwrap().get_mut(*part).unwrap();
        }
    }
    
    Ok(())
}

fn create_config_file(bl_dir: &Path, args: &InitArgs) -> Result<()> {
    let config_content = format!(
        r#"[repository]
name = "{}"
description = "{}"
author = "{}"
license = "{}"
version = "{}"
tags = ["{}"]

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
"#,
        args.namespace,
        "Machine Learning Pipeline",
        args.owner.as_deref().unwrap_or("Data Science Team"),
        "MIT",
        "1.0.0",
        "ml,ai,pipeline"
    );
    
    fs::write(bl_dir.join("config.toml"), config_content)?;
    println!("📄 Created config.toml");
    Ok(())
}

fn create_metadata_template(bl_dir: &Path, args: &InitArgs) -> Result<()> {
    let metadata = serde_json::json!({
        "repository": {
            "name": args.namespace,
            "description": "Machine Learning Pipeline",
            "author": args.owner.as_deref().unwrap_or("Data Science Team"),
            "license": "MIT",
            "version": "1.0.0",
            "tags": ["ml", "ai", "pipeline"],
            "created_at": OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap(),
            "updated_at": OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339).unwrap()
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
    });
    
    // Apply dot notation overrides
    let mut json_value = metadata;
    for (key, value) in &args.set {
        set_nested_value(&mut json_value, key, value)?;
    }
    
    let metadata_content = serde_json::to_string_pretty(&json_value)?;
    fs::write(bl_dir.join("metadata.json"), metadata_content)?;
    println!("📄 Created metadata.json");
    Ok(())
}

fn create_gitignore(bl_dir: &Path) -> Result<()> {
    let gitignore_content = r#"# BlackLake specific ignores
.bl/objects/
.bl/refs/
.bl/hooks/

# Temporary files
*.tmp
*.temp
*.log

# OS specific
.DS_Store
Thumbs.db

# IDE specific
.vscode/
.idea/
*.swp
*.swo

# Python
__pycache__/
*.pyc
*.pyo
*.pyd
.Python
env/
venv/
.venv/

# R
.Rhistory
.RData
.Ruserdata

# Jupyter
.ipynb_checkpoints/

# Data files (uncomment as needed)
# *.csv
# *.parquet
# *.h5
# *.hdf5
"#;
    
    fs::write(bl_dir.join(".gitignore"), gitignore_content)?;
    println!("📄 Created .gitignore");
    Ok(())
}

fn create_hooks(bl_dir: &Path) -> Result<()> {
    let hooks_dir = bl_dir.join("hooks");
    
    // Pre-commit hook
    let pre_commit_content = r#"#!/bin/bash
# BlackLake pre-commit hook
echo "🔍 Running BlackLake pre-commit checks..."

# Validate metadata
blacklake-cli validate

# Check file permissions
find . -name "*.bl" -exec chmod 600 {} \;

echo "✅ Pre-commit checks passed"
"#;
    fs::write(hooks_dir.join("pre-commit"), pre_commit_content)?;
    
    // Post-commit hook
    let post_commit_content = r#"#!/bin/bash
# BlackLake post-commit hook
echo "📝 Running BlackLake post-commit actions..."

# Update metadata timestamps
blacklake-cli update-metadata

# Generate provenance
blacklake-cli generate-provenance

echo "✅ Post-commit actions completed"
"#;
    fs::write(hooks_dir.join("post-commit"), post_commit_content)?;
    
    // Pre-push hook
    let pre_push_content = r#"#!/bin/bash
# BlackLake pre-push hook
echo "🚀 Running BlackLake pre-push checks..."

# Validate repository state
blacklake-cli validate-repo

# Check for conflicts
blacklake-cli check-conflicts

echo "✅ Pre-push checks passed"
"#;
    fs::write(hooks_dir.join("pre-push"), pre_push_content)?;
    
    // Make hooks executable
    #[cfg(unix)]
    {
        for hook in &["pre-commit", "post-commit", "pre-push"] {
            let hook_path = hooks_dir.join(hook);
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }
    }
    
    println!("📄 Created hooks");
    Ok(())
}