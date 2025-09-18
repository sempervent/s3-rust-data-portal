use anyhow::{anyhow, Result};
use blacklake_core::{CanonicalMeta, Change, ChangeOp, CommitRequest};
use crate::api::ApiClient;
use crate::prompt::{collect_metadata_interactive, PromptContext};
use clap::Args;
use colored::*;
use serde_json::Value;
use similar::{ChangeTag, TextDiff};
use std::path::Path;
use std::process::Command;

#[derive(Args)]
pub struct MetaEditArgs {
    /// Repository name
    pub repo: String,
    
    /// Branch or ref name
    pub r#ref: String,
    
    /// File path in repository
    pub path: String,
    
    /// Open editor for metadata
    #[arg(long)]
    pub open_editor: bool,
    
    /// Metadata as JSON/YAML file
    #[arg(long)]
    pub meta: Option<String>,
    
    /// Metadata key-value pairs
    #[arg(long, value_parser = parse_key_value)]
    pub meta_key: Vec<(String, String)>,
    
    /// Dry run (don't commit)
    #[arg(long)]
    pub dry_run: bool,
}

fn parse_key_value(s: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err(anyhow!("Invalid key=value format: {}", s));
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

pub async fn meta_edit_command(args: MetaEditArgs, api_client: &ApiClient) -> Result<()> {
    println!("📝 Editing metadata for {}/{}", args.repo.blue(), args.path.blue());

    // Get current metadata
    let current_metadata = get_current_metadata(&args, api_client).await?;
    
    // Collect new metadata
    let new_metadata = if args.open_editor {
        edit_metadata_with_editor(&current_metadata)?
    } else if args.meta.is_some() || !args.meta_key.is_empty() {
        collect_metadata_from_args(&args, &current_metadata)?
    } else {
        collect_metadata_interactive(&PromptContext {
            file_path: args.path.clone(),
            file_size: current_metadata.file_size as u64,
            mime_type: Some(current_metadata.file_type.clone()),
            user_email: None, // TODO: Get from OIDC token
            template: None,
        })?
    };

    // Show diff
    println!("📊 Metadata changes:");
    show_metadata_diff(&current_metadata, &new_metadata);

    if args.dry_run {
        println!("🔍 Dry run - would commit metadata changes");
        return Ok(());
    }

    // Create metadata-only commit
    let commit_request = CommitRequest {
        ref: args.r#ref.clone(),
        message: format!("Update metadata for {}", args.path),
        changes: vec![Change {
            op: ChangeOp::Meta,
            path: args.path,
            sha256: None,
            meta: serde_json::to_value(&new_metadata)?,
        }],
    };

    println!("💾 Committing metadata changes...");
    let commit_response = api_client.commit(&args.repo, &commit_request, true).await?;

    println!("✅ Successfully updated metadata: {}", commit_response.commit_id);
    Ok(())
}

async fn get_current_metadata(args: &MetaEditArgs, api_client: &ApiClient) -> Result<CanonicalMeta> {
    // Get the current tree to find the file
    let tree_response = api_client.get_tree(&args.repo, &args.r#ref, Some(&args.path)).await?;
    
    let entry = tree_response.entries.iter()
        .find(|e| e.path == args.path)
        .ok_or_else(|| anyhow!("File not found: {}", args.path))?;

    // Parse existing metadata or create default
    let metadata = if let Some(ref meta) = entry.meta {
        serde_json::from_value(meta.clone())?
    } else {
        CanonicalMeta {
            creation_dt: chrono::Utc::now().to_rfc3339(),
            creator: "unknown".to_string(),
            file_name: Path::new(&args.path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            file_type: "application/octet-stream".to_string(),
            file_size: entry.size as i64,
            org_lab: "Unknown".to_string(),
            description: "No description".to_string(),
            data_source: "unknown".to_string(),
            data_collection_method: "unknown".to_string(),
            version: "1.0".to_string(),
            notes: None,
            tags: None,
            license: None,
        }
    };

    Ok(metadata)
}

fn edit_metadata_with_editor(current_metadata: &CanonicalMeta) -> Result<CanonicalMeta> {
    let temp_file = std::env::temp_dir().join("blacklake_metadata.yaml");
    
    // Write current metadata to temp file
    let yaml_content = serde_yaml::to_string(current_metadata)?;
    std::fs::write(&temp_file, yaml_content)?;

    // Open editor
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let status = Command::new(&editor)
        .arg(&temp_file)
        .status()?;

    if !status.success() {
        return Err(anyhow!("Editor exited with error"));
    }

    // Read edited metadata
    let edited_content = std::fs::read_to_string(&temp_file)?;
    let edited_metadata: CanonicalMeta = serde_yaml::from_str(&edited_content)?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_file);

    Ok(edited_metadata)
}

fn collect_metadata_from_args(args: &MetaEditArgs, current_metadata: &CanonicalMeta) -> Result<CanonicalMeta> {
    let mut metadata = current_metadata.clone();

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
    }
}
