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
    let mime_type = args.r#type.or_else(|| {
        from_path(local_file_path).first().map(|m| m.to_string())
    });

    println!("🚀 Uploading {} to {}/{}", 
        args.local_file.green(), 
        args.repo.blue(), 
        args.path.blue()
    );

    // Step 1: Initialize upload
    let upload_init = api_client.upload_init(&args.repo, &crate::api::UploadInitRequest {
        path: args.path.clone(),
        size: file_size,
        media_type: mime_type.clone(),
    }).await?;

    println!("📤 Uploading file...");
    api_client.upload_file(&upload_init.upload_url, local_file_path).await?;

    // Step 2: Collect metadata
    let metadata = if args.non_interactive {
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
        ref: args.r#ref,
        message: format!("Add {}", args.path),
        changes: vec![Change {
            op: ChangeOp::Add,
            path: args.path,
            sha256: Some(upload_init.sha256),
            meta: serde_json::to_value(&metadata)?,
        }],
    };

    println!("💾 Committing changes...");
    let commit_response = api_client.commit(&args.repo, &commit_request, true).await?;

    println!("✅ Successfully committed: {}", commit_response.commit_id);
    
    if args.emit_rdf {
        println!("🔗 RDF metadata available at: /v1/repos/{}/rdf/{}/{}", 
            args.repo, args.r#ref, args.path);
    }

    Ok(())
}

fn collect_metadata_non_interactive(args: &PutArgs) -> Result<CanonicalMeta> {
    let mut metadata = CanonicalMeta {
        creation_dt: chrono::Utc::now().to_rfc3339(),
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
        "creation_dt" => metadata.creation_dt = value.to_string(),
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

pub fn show_metadata_diff(old: &CanonicalMeta, new: &CanonicalMeta) {
    let old_json = serde_json::to_string_pretty(old).unwrap_or_default();
    let new_json = serde_json::to_string_pretty(new).unwrap_or_default();
    
    let diff = TextDiff::from_lines(&old_json, &new_json);
    
    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            println!("{:-^1$}", "-", 80);
        }
        for op in group {
            for change in diff.iter_inline_changes(op) {
                let (sign, style) = match change.tag() {
                    ChangeTag::Delete => ("-", "red"),
                    ChangeTag::Insert => ("+", "green"),
                    ChangeTag::Equal => (" ", "white"),
                };
                print!("{}{}", style.apply_to(sign).bold(), style.apply_to(change));
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
