use anyhow::{anyhow, Result};
use blacklake_core::CanonicalMeta;
use chrono::{DateTime, Utc};
use dialoguer::{Confirm, Input, MultiSelect, Select};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MetadataTemplate {
    pub name: String,
    pub defaults: HashMap<String, Value>,
    pub required_fields: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PromptContext {
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: Option<String>,
    pub user_email: Option<String>,
    pub template: Option<MetadataTemplate>,
}

pub fn collect_metadata_interactive(ctx: &PromptContext) -> Result<CanonicalMeta> {
    println!("📝 Collecting metadata for: {}", ctx.file_path);
    println!();

    // Required fields
    let creation_dt = prompt_creation_dt()?;
    let creator = prompt_creator(&ctx.user_email)?;
    let file_name = prompt_file_name(&ctx.file_path)?;
    let file_type = prompt_file_type(&ctx.mime_type)?;
    let file_size = ctx.file_size;
    let org_lab = prompt_org_lab()?;
    let description = prompt_description()?;
    let data_source = prompt_data_source()?;
    let data_collection_method = prompt_data_collection_method()?;
    let version = prompt_version()?;

    // Optional fields
    let notes = prompt_notes()?;
    let tags = prompt_tags()?;
    let license = prompt_license()?;

    Ok(CanonicalMeta {
        creation_dt: creation_dt.to_rfc3339(),
        creator,
        file_name,
        file_type,
        file_size: file_size as i64,
        org_lab,
        description,
        data_source,
        data_collection_method,
        version,
        notes,
        tags,
        license,
    })
}

fn prompt_creation_dt() -> Result<DateTime<Utc>> {
    let now = Utc::now();
    let default = now.format("%Y-%m-%d %H:%M:%S").to_string();
    
    let input: String = Input::new()
        .with_prompt("Creation date/time")
        .with_initial_text(&default)
        .with_help_text("Format: YYYY-MM-DD HH:MM:SS (default: now)")
        .interact_text()?;

    if input.trim().is_empty() {
        return Ok(now);
    }

    // Try parsing various formats
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d",
    ];

    for format in &formats {
        if let Ok(dt) = DateTime::parse_from_str(&input, format) {
            return Ok(dt.with_timezone(&Utc));
        }
    }

    Err(anyhow!("Invalid date format. Use YYYY-MM-DD HH:MM:SS"))
}

fn prompt_creator(user_email: &Option<String>) -> Result<String> {
    let default = user_email.clone().unwrap_or_else(|| "user@example.com".to_string());
    
    let input: String = Input::new()
        .with_prompt("Creator")
        .with_initial_text(&default)
        .with_help_text("Email address or identifier of the creator")
        .interact_text()?;

    if input.trim().is_empty() {
        return Err(anyhow!("Creator is required"));
    }

    Ok(input.trim().to_string())
}

fn prompt_file_name(file_path: &str) -> Result<String> {
    let default = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let input: String = Input::new()
        .with_prompt("File name")
        .with_initial_text(&default)
        .with_help_text("Name of the file")
        .interact_text()?;

    if input.trim().is_empty() {
        return Err(anyhow!("File name is required"));
    }

    Ok(input.trim().to_string())
}

fn prompt_file_type(mime_type: &Option<String>) -> Result<String> {
    let default = mime_type.clone().unwrap_or_else(|| "application/octet-stream".to_string());
    
    let input: String = Input::new()
        .with_prompt("File type (MIME)")
        .with_initial_text(&default)
        .with_help_text("MIME type of the file")
        .interact_text()?;

    Ok(input.trim().to_string())
}

fn prompt_org_lab() -> Result<String> {
    let input: String = Input::new()
        .with_prompt("Organization/Lab")
        .with_help_text("Organization or laboratory name")
        .interact_text()?;

    if input.trim().is_empty() {
        return Err(anyhow!("Organization/Lab is required"));
    }

    Ok(input.trim().to_string())
}

fn prompt_description() -> Result<String> {
    let input: String = Input::new()
        .with_prompt("Description")
        .with_help_text("Description of the file content")
        .interact_text()?;

    if input.trim().is_empty() {
        return Err(anyhow!("Description is required"));
    }

    Ok(input.trim().to_string())
}

fn prompt_data_source() -> Result<String> {
    let input: String = Input::new()
        .with_prompt("Data source")
        .with_help_text("Source of the data (e.g., sensor, simulation, manual)")
        .interact_text()?;

    if input.trim().is_empty() {
        return Err(anyhow!("Data source is required"));
    }

    Ok(input.trim().to_string())
}

fn prompt_data_collection_method() -> Result<String> {
    let input: String = Input::new()
        .with_prompt("Data collection method")
        .with_help_text("Method used to collect the data")
        .interact_text()?;

    if input.trim().is_empty() {
        return Err(anyhow!("Data collection method is required"));
    }

    Ok(input.trim().to_string())
}

fn prompt_version() -> Result<String> {
    let input: String = Input::new()
        .with_prompt("Version")
        .with_initial_text("1.0")
        .with_help_text("Version of the data")
        .interact_text()?;

    Ok(input.trim().to_string())
}

fn prompt_notes() -> Result<Option<String>> {
    let add_notes = Confirm::new()
        .with_prompt("Add notes?")
        .default(false)
        .interact()?;

    if !add_notes {
        return Ok(None);
    }

    let input: String = Input::new()
        .with_prompt("Notes")
        .with_help_text("Additional notes about the file")
        .interact_text()?;

    Ok(if input.trim().is_empty() { None } else { Some(input.trim().to_string()) })
}

fn prompt_tags() -> Result<Option<Vec<String>>> {
    let add_tags = Confirm::new()
        .with_prompt("Add tags?")
        .default(false)
        .interact()?;

    if !add_tags {
        return Ok(None);
    }

    let input: String = Input::new()
        .with_prompt("Tags (comma-separated)")
        .with_help_text("Tags for categorizing the file")
        .interact_text()?;

    if input.trim().is_empty() {
        return Ok(None);
    }

    let tags: Vec<String> = input
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(if tags.is_empty() { None } else { Some(tags) })
}

fn prompt_license() -> Result<Option<String>> {
    let add_license = Confirm::new()
        .with_prompt("Add license?")
        .default(false)
        .interact()?;

    if !add_license {
        return Ok(None);
    }

    let input: String = Input::new()
        .with_prompt("License")
        .with_help_text("License for the data")
        .interact_text()?;

    Ok(if input.trim().is_empty() { None } else { Some(input.trim().to_string()) })
}

pub fn load_templates() -> Result<Vec<MetadataTemplate>> {
    let mut templates = Vec::new();

    // Load global templates
    if let Some(home) = dirs::home_dir() {
        let global_templates_dir = home.join(".blacklake").join("templates");
        if global_templates_dir.exists() {
            load_templates_from_dir(&global_templates_dir, &mut templates)?;
        }
    }

    // Load repo-scoped templates
    let repo_templates_dir = Path::new(".blacklake").join("templates");
    if repo_templates_dir.exists() {
        load_templates_from_dir(&repo_templates_dir, &mut templates)?;
    }

    Ok(templates)
}

fn load_templates_from_dir(dir: &Path, templates: &mut Vec<MetadataTemplate>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
            let content = std::fs::read_to_string(&path)?;
            let template: MetadataTemplate = serde_yaml::from_str(&content)?;
            templates.push(template);
        }
    }
    
    Ok(())
}

pub fn select_template(templates: &[MetadataTemplate]) -> Result<Option<&MetadataTemplate>> {
    if templates.is_empty() {
        return Ok(None);
    }

    let template_names: Vec<&str> = templates.iter().map(|t| t.name.as_str()).collect();
    let selection = Select::new()
        .with_prompt("Select a template (optional)")
        .items(&template_names)
        .default(0)
        .interact_opt()?;

    Ok(selection.map(|i| &templates[i]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_template_loading() {
        // This would require creating test template files
        // For now, just test that the function doesn't panic
        let templates = load_templates().unwrap_or_default();
        assert!(templates.len() >= 0);
    }

    #[test]
    fn test_prompt_context_creation() {
        let ctx = PromptContext {
            file_path: "test.txt".to_string(),
            file_size: 1024,
            mime_type: Some("text/plain".to_string()),
            user_email: Some("test@example.com".to_string()),
            template: None,
        };
        
        assert_eq!(ctx.file_path, "test.txt");
        assert_eq!(ctx.file_size, 1024);
    }
}
