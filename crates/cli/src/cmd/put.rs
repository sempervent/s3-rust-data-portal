use anyhow::{anyhow, Result};
use blacklake_core::{CanonicalMeta, Change, ChangeOp, CommitRequest};
use crate::api::ApiClient;
use crate::prompt::{collect_metadata_interactive, load_templates, select_template, PromptContext};
use clap::Args;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use mime_guess::from_path;
use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
// Remove BlackLakeMetadata import - it doesn't exist in the new init module

#[derive(Args)]
pub struct PutArgs {
    /// Repository name
    pub repo: String,
    
    /// Branch or ref name
    pub r#ref: String,
    
    /// Local file path
    pub local_file: String,
    
    /// Logical path in repository
    #[arg(long)]
    pub path: String,
    
    /// MIME type (auto-detected if not specified)
    #[arg(long)]
    pub r#type: Option<String>,
    
    /// Emit RDF metadata
    #[arg(long)]
    pub emit_rdf: bool,
    
    /// Open editor for metadata
    #[arg(long)]
    pub open_editor: bool,
    
    /// Metadata as JSON/YAML
    #[arg(long)]
    pub meta: Option<String>,
    
    /// Metadata key-value pairs
    #[arg(long, value_parser = parse_key_value)]
    pub meta_key: Vec<(String, String)>,
    
    /// Template name
    #[arg(long)]
    pub template: Option<String>,
    
    /// Dry run (don't commit)
    #[arg(long)]
    pub dry_run: bool,
    
    /// Non-interactive mode
    #[arg(long)]
    pub non_interactive: bool,
}

fn parse_key_value(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid key=value format: {}", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub async fn put_command(args: PutArgs, api_client: &ApiClient) -> Result<()> {
    let local_file_path = Path::new(&args.local_file);
    if !local_file_path.exists() {
        return Err(anyhow!("File not found: {}", args.local_file));
    }

    let file_size = std::fs::metadata(local_file_path)?.len();
    let mime_type = args.r#type.clone().or_else(|| {
        from_path(local_file_path).first().map(|m| m.to_string())
    });

    println!("🚀 Uploading {} to {}/{}", 
        args.local_file.green(), 
        args.repo.blue(), 
        args.path.blue()
    );

    // Check for BlackLake metadata files
    let bl_metadata = detect_blacklake_metadata(local_file_path)?;
    if let Some(metadata) = &bl_metadata {
        println!("📋 Found BlackLake metadata: {}", 
            if local_file_path.is_file() {
                format!("{}.bl.metadata.yaml", args.local_file)
            } else {
                format!("{}/.bl/{}.metadata.yaml", args.local_file, 
                    Path::new(&args.path).file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown"))
            }
        );
    }

    // Step 1: Initialize upload
    let upload_init = api_client.upload_init(&args.repo, &crate::api::UploadInitRequest {
        path: args.path.clone(),
        size: file_size,
        media_type: mime_type.clone(),
    }).await?;

    println!("📤 Uploading file...");
    api_client.upload_file(&upload_init.upload_url, local_file_path).await?;

    // Step 2: Collect metadata
    let metadata = if let Some(bl_metadata) = bl_metadata {
        // Use BlackLake metadata if found
        convert_blacklake_metadata_to_canonical(&bl_metadata, &args.path, file_size, mime_type)?
    } else if args.non_interactive {
        collect_metadata_non_interactive(&args)?
    } else {
        collect_metadata_interactive(&PromptContext {
            file_path: args.path.clone(),
            file_size,
            mime_type,
            user_email: None, // TODO: Get from OIDC token
            template: None,
        })?
    };

    // If uploading a directory, handle multiple files
    if local_file_path.is_dir() {
        return upload_directory_with_metadata(local_file_path, &args, api_client, &metadata).await;
    }

    // Step 3: Create commit
    if args.dry_run {
        println!("🔍 Dry run - would commit:");
        println!("  Repository: {}", args.repo);
        println!("  Ref: {}", args.r#ref);
        println!("  Path: {}", args.path);
        println!("  SHA256: {}", upload_init.sha256);
        println!("  Metadata: {}", serde_json::to_string_pretty(&metadata)?);
        return Ok(());
    }

    let commit_request = CommitRequest {
        r#ref: args.r#ref.clone(),
        message: Some(format!("Add {}", args.path)),
        expected_parent: None,
        changes: vec![Change {
            op: ChangeOp::Add,
            path: args.path.clone(),
            sha256: Some(upload_init.sha256),
            meta: serde_json::to_value(&metadata)?,
        }],
    };

    println!("💾 Committing changes...");
    let commit_response = api_client.commit(&args.repo, &commit_request, true).await?;

    println!("✅ Successfully committed: {:?}", commit_response.commit_id);
    
    if args.emit_rdf {
        println!("🔗 RDF metadata available at: /v1/repos/{}/rdf/{}/{}", 
            args.repo, args.r#ref, args.path);
    }

    Ok(())
}

fn collect_metadata_non_interactive(args: &PutArgs) -> Result<CanonicalMeta> {
    let mut metadata = CanonicalMeta {
        creation_dt: chrono::Utc::now(),
        creator: "cli-user".to_string(),
        file_name: Path::new(&args.path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string(),
        file_type: args.r#type.clone().unwrap_or_else(|| "application/octet-stream".to_string()),
        file_size: 0, // Will be set by caller
        org_lab: "Unknown".to_string(),
        description: "Uploaded via CLI".to_string(),
        data_source: "cli".to_string(),
        data_collection_method: "upload".to_string(),
        version: "1.0".to_string(),
        notes: None,
        tags: None,
        license: None,
    };

    // Apply metadata from --meta file
    if let Some(ref meta_file) = args.meta {
        let meta_content = std::fs::read_to_string(meta_file)?;
        let meta_value: Value = if meta_file.ends_with(".yaml") || meta_file.ends_with(".yml") {
            serde_yaml::from_str(&meta_content)?
        } else {
            serde_json::from_str(&meta_content)?
        };
        
        // Merge with existing metadata
        merge_metadata(&mut metadata, &meta_value)?;
    }

    // Apply metadata from --meta-key flags
    for (key, value) in &args.meta_key {
        set_metadata_field(&mut metadata, key, value)?;
    }

    // Apply template
    if let Some(ref template_name) = args.template {
        apply_template(&mut metadata, template_name)?;
    }

    Ok(metadata)
}

fn merge_metadata(metadata: &mut CanonicalMeta, meta_value: &Value) -> Result<()> {
    if let Some(obj) = meta_value.as_object() {
        for (key, value) in obj {
            if let Some(str_value) = value.as_str() {
                set_metadata_field(metadata, key, str_value)?;
            }
        }
    }
    Ok(())
}

fn set_metadata_field(metadata: &mut CanonicalMeta, key: &str, value: &str) -> Result<()> {
    match key {
        "creation_dt" => {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&value) {
                metadata.creation_dt = dt.with_timezone(&chrono::Utc);
            }
        },
        "creator" => metadata.creator = value.to_string(),
        "file_name" => metadata.file_name = value.to_string(),
        "file_type" => metadata.file_type = value.to_string(),
        "org_lab" => metadata.org_lab = value.to_string(),
        "description" => metadata.description = value.to_string(),
        "data_source" => metadata.data_source = value.to_string(),
        "data_collection_method" => metadata.data_collection_method = value.to_string(),
        "version" => metadata.version = value.to_string(),
        "notes" => metadata.notes = Some(value.to_string()),
        "license" => metadata.license = Some(value.to_string()),
        "tags" => {
            let tags: Vec<String> = value.split(',').map(|s| s.trim().to_string()).collect();
            metadata.tags = Some(tags);
        },
        _ => return Err(anyhow!("Unknown metadata field: {}", key)),
    }
    Ok(())
}

fn apply_template(metadata: &mut CanonicalMeta, template_name: &str) -> Result<()> {
    let templates = load_templates()?;
    let template = templates.iter()
        .find(|t| t.name == template_name)
        .ok_or_else(|| anyhow!("Template not found: {}", template_name))?;

    // Apply template defaults
    for (key, value) in &template.defaults {
        if let Some(str_value) = value.as_str() {
            set_metadata_field(metadata, key, str_value)?;
        }
    }

    Ok(())
}

fn detect_blacklake_metadata(local_file_path: &Path) -> Result<Option<serde_json::Value>> {
    if local_file_path.is_file() {
        // Check for {filename}.bl.metadata.yaml
        let metadata_path = local_file_path.with_extension("bl.metadata.yaml");
        if metadata_path.exists() {
            return load_blacklake_metadata(&metadata_path);
        }
    } else if local_file_path.is_dir() {
        // Check for .bl/{filename}.metadata.yaml
        let bl_dir = local_file_path.join(".bl");
        if bl_dir.exists() {
            // For directories, we'll look for a generic directory metadata
            let dir_metadata_path = bl_dir.join("directory.metadata.yaml");
            if dir_metadata_path.exists() {
                return load_blacklake_metadata(&dir_metadata_path);
            }
        }
    }
    Ok(None)
}

fn load_blacklake_metadata(metadata_path: &Path) -> Result<Option<serde_json::Value>> {
    let content = std::fs::read_to_string(metadata_path)?;
    let metadata: serde_json::Value = serde_yaml::from_str(&content)?;
    Ok(Some(metadata))
}

fn convert_blacklake_metadata_to_canonical(
    bl_metadata: &serde_json::Value,
    path: &str,
    file_size: u64,
    mime_type: Option<String>,
) -> Result<CanonicalMeta> {
    let file_name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let mut canonical = CanonicalMeta {
        creation_dt: chrono::Utc::now(),
        creator: bl_metadata.get("provenance")
            .and_then(|p| p.get("author"))
            .and_then(|a| a.get("email"))
            .and_then(|e| e.as_str())
            .unwrap_or("unknown@example.com")
            .to_string(),
        file_name,
        file_type: mime_type.unwrap_or_else(|| {
            bl_metadata.get("artifact")
                .and_then(|a| a.get("mime_type"))
                .and_then(|m| m.as_str())
                .unwrap_or("application/octet-stream")
                .to_string()
        }),
        file_size: file_size as i64,
        org_lab: bl_metadata.get("provenance")
            .and_then(|p| p.get("organization"))
            .and_then(|o| o.as_str())
            .unwrap_or("Unknown Organization")
            .to_string(),
        description: bl_metadata.get("artifact")
            .and_then(|a| a.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("BlackLake artifact")
            .to_string(),
        data_source: "blacklake-cli".to_string(),
        data_collection_method: "upload".to_string(),
        version: bl_metadata.get("artifact")
            .and_then(|a| a.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string(),
        notes: None,
        tags: bl_metadata.get("artifact")
            .and_then(|a| a.get("tags"))
            .and_then(|t| t.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect()),
        license: bl_metadata.get("provenance")
            .and_then(|p| p.get("license"))
            .and_then(|l| l.as_str())
            .map(|s| s.to_string()),
    };

    // Parse creation date from BlackLake metadata
    if let Some(created_at) = bl_metadata.get("technical")
        .and_then(|t| t.get("created_at"))
        .and_then(|c| c.as_str()) {
        if let Ok(parsed_dt) = chrono::DateTime::parse_from_rfc3339(created_at) {
            canonical.creation_dt = parsed_dt.with_timezone(&chrono::Utc);
        }
    }

    // Add custom fields as notes
    if let Some(custom) = bl_metadata.get("custom") {
        if !custom.is_null() {
            let custom_json = serde_json::to_string_pretty(custom)?;
            canonical.notes = Some(format!("Custom metadata:\n{}", custom_json));
        }
    }

    // Add authorization info to notes
    let auth_info = format!(
        "Authorization: {} ({}), Legal Hold: {}, Retention: {}",
        bl_metadata.get("authorization")
            .and_then(|a| a.get("access_level"))
            .and_then(|l| l.as_str())
            .unwrap_or("restricted"),
        bl_metadata.get("authorization")
            .and_then(|a| a.get("data_classification"))
            .and_then(|c| c.as_str())
            .unwrap_or("confidential"),
        bl_metadata.get("authorization")
            .and_then(|a| a.get("legal_hold"))
            .and_then(|h| h.as_bool())
            .unwrap_or(false),
        bl_metadata.get("authorization")
            .and_then(|a| a.get("retention_policy"))
            .and_then(|p| p.as_str())
            .unwrap_or("indefinite")
    );
    
    if let Some(ref mut notes) = canonical.notes {
        notes.push_str(&format!("\n{}", auth_info));
    } else {
        canonical.notes = Some(auth_info);
    }

    Ok(canonical)
}

async fn upload_directory_with_metadata(
    dir_path: &Path,
    args: &PutArgs,
    api_client: &ApiClient,
    base_metadata: &CanonicalMeta,
) -> Result<()> {
    println!("📁 Uploading directory with metadata...");
    
    let bl_dir = dir_path.join(".bl");
    let mut changes = Vec::new();
    
    // Walk through directory and upload each file
    let entries = std::fs::read_dir(dir_path)?;
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        
        // Skip .bl directory and hidden files
        if entry_path.file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.starts_with('.') || s == ".bl")
            .unwrap_or(false) {
            continue;
        }
        
        if entry_path.is_file() {
            let relative_path = entry_path.strip_prefix(dir_path)
                .unwrap_or(&entry_path);
            let repo_path = format!("{}/{}", args.path.trim_end_matches('/'), 
                relative_path.to_string_lossy().replace('\\', "/"));
            
            // Check for individual file metadata
            let file_metadata = if bl_dir.exists() {
                let metadata_file = bl_dir.join(format!("{}.metadata.yaml", 
                    entry_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")));
                
                if metadata_file.exists() {
                    if let Ok(Some(bl_metadata)) = load_blacklake_metadata(&metadata_file) {
                        let file_size = std::fs::metadata(&entry_path)?.len();
                        let mime_type = from_path(&entry_path).first().map(|m| m.to_string());
                        convert_blacklake_metadata_to_canonical(&bl_metadata, &repo_path, file_size, mime_type).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };
            
            // Use file-specific metadata or fall back to base metadata
            let metadata = file_metadata.unwrap_or_else(|| {
                let mut meta = base_metadata.clone();
                meta.file_name = entry_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                meta.file_size = std::fs::metadata(&entry_path).map(|m| m.len() as i64).unwrap_or(0);
                meta
            });
            
            // Upload file
            let file_size = std::fs::metadata(&entry_path)?.len();
            let mime_type = from_path(&entry_path).first().map(|m| m.to_string());
            
            let upload_init = api_client.upload_init(&args.repo, &crate::api::UploadInitRequest {
                path: repo_path.clone(),
                size: file_size,
                media_type: mime_type,
            }).await?;
            
            api_client.upload_file(&upload_init.upload_url, &entry_path).await?;
            
            changes.push(Change {
                op: ChangeOp::Add,
                path: repo_path,
                sha256: Some(upload_init.sha256),
                meta: serde_json::to_value(&metadata)?,
            });
            
            println!("  📄 Uploaded: {}", entry_path.display());
        }
    }
    
    if changes.is_empty() {
        return Err(anyhow!("No files found to upload in directory"));
    }
    
    // Create commit with all changes
    let commit_request = CommitRequest {
        r#ref: args.r#ref.clone(),
        message: Some(format!("Add directory {} with {} files", args.path, changes.len())),
        expected_parent: None,
        changes,
    };
    
    println!("💾 Committing {} files...", commit_request.changes.len());
    let commit_response = api_client.commit(&args.repo, &commit_request, true).await?;
    
    println!("✅ Successfully committed directory: {:?}", commit_response.commit_id);
    
    if args.emit_rdf {
        println!("🔗 RDF metadata available for each file at: /v1/repos/{}/rdf/{}/<path>", 
            args.repo, args.r#ref);
    }
    
    Ok(())
}

pub fn show_metadata_diff(old: &CanonicalMeta, new: &CanonicalMeta) {
    let old_json = serde_json::to_string_pretty(old).unwrap_or_default();
    let new_json = serde_json::to_string_pretty(new).unwrap_or_default();
    
    let diff = TextDiff::from_lines(&old_json, &new_json);
    
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{:-^1$}", "-", 80);
        }
        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", "red"),
                    ChangeTag::Insert => ("+", "green"),
                    ChangeTag::Equal => (" ", "white"),
                };
                let colored_sign = match style {
                    "red" => colored::Colorize::red(sign),
                    "green" => colored::Colorize::green(sign),
                    _ => colored::Colorize::white(sign),
                };
                let change_str = change.to_string();
                let colored_change = match style {
                    "red" => colored::Colorize::red(&*change_str),
                    "green" => colored::Colorize::green(&*change_str),
                    _ => colored::Colorize::white(&*change_str),
                };
                print!("{}{}", colored_sign.bold(), colored_change);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_value() {
        assert_eq!(parse_key_value("key=value").unwrap(), ("key".to_string(), "value".to_string()));
        assert_eq!(parse_key_value("key=value=with=equals").unwrap(), ("key".to_string(), "value=with=equals".to_string()));
    }

    #[test]
    fn test_parse_key_value_invalid() {
        assert!(parse_key_value("key").is_err());
        assert!(parse_key_value("").is_err());
    }

    #[test]
    fn test_set_metadata_field() {
        let mut metadata = CanonicalMeta {
            creation_dt: "".to_string(),
            creator: "".to_string(),
            file_name: "".to_string(),
            file_type: "".to_string(),
            file_size: 0,
            org_lab: "".to_string(),
            description: "".to_string(),
            data_source: "".to_string(),
            data_collection_method: "".to_string(),
            version: "".to_string(),
            notes: None,
            tags: None,
            license: None,
        };

        set_metadata_field(&mut metadata, "creator", "test@example.com").unwrap();
        assert_eq!(metadata.creator, "test@example.com");

        set_metadata_field(&mut metadata, "tags", "tag1,tag2,tag3").unwrap();
        assert_eq!(metadata.tags, Some(vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()]));
    }
}
