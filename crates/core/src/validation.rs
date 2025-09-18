use anyhow::{anyhow, Result};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;

/// Repository name validation
pub fn validate_repo_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(anyhow!("Repository name cannot be empty"));
    }

    if name.len() > 100 {
        return Err(anyhow!("Repository name too long (max 100 characters)"));
    }

    // Allow alphanumeric, hyphens, underscores, and dots
    let repo_name_regex = Regex::new(r"^[a-zA-Z0-9._-]+$")?;
    if !repo_name_regex.is_match(name) {
        return Err(anyhow!(
            "Repository name can only contain alphanumeric characters, hyphens, underscores, and dots"
        ));
    }

    // Cannot start or end with dot or hyphen
    if name.starts_with('.') || name.starts_with('-') || name.ends_with('.') || name.ends_with('-') {
        return Err(anyhow!(
            "Repository name cannot start or end with dot or hyphen"
        ));
    }

    // Cannot have consecutive dots
    if name.contains("..") {
        return Err(anyhow!("Repository name cannot contain consecutive dots"));
    }

    Ok(())
}

/// Path normalization and validation
pub fn normalize_path(path: &str) -> Result<String> {
    if path.is_empty() {
        return Err(anyhow!("Path cannot be empty"));
    }

    // Remove leading/trailing slashes
    let mut normalized = path.trim_matches('/').to_string();

    if normalized.is_empty() {
        return Err(anyhow!("Path cannot be empty after normalization"));
    }

    // Check for path traversal attempts
    if normalized.contains("..") {
        return Err(anyhow!("Path traversal not allowed"));
    }

    // Check for null bytes
    if normalized.contains('\0') {
        return Err(anyhow!("Path cannot contain null bytes"));
    }

    // Normalize path separators
    normalized = normalized.replace('\\', "/");

    // Remove duplicate slashes
    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    Ok(normalized)
}

/// Metadata validation against schema
pub fn validate_meta(meta: &Value, schema_version: Option<&str>) -> Result<()> {
    let schema_version = schema_version.unwrap_or("1.0");
    
    match schema_version {
        "1.0" => validate_meta_v1_0(meta),
        "dublin-core" => validate_dublin_core_meta(meta),
        _ => Err(anyhow!("Unsupported schema version: {}", schema_version)),
    }
}

fn validate_meta_v1_0(meta: &Value) -> Result<()> {
    let obj = meta.as_object()
        .ok_or_else(|| anyhow!("Metadata must be a JSON object"))?;

    // Required fields
    let required_fields = ["name"];
    for field in &required_fields {
        if !obj.contains_key(*field) {
            return Err(anyhow!("Missing required field: {}", field));
        }
        
        if obj[*field].as_str().map(|s| s.is_empty()).unwrap_or(true) {
            return Err(anyhow!("Field '{}' cannot be empty", field));
        }
    }

    // Validate field types and constraints
    if let Some(name) = obj.get("name") {
        if let Some(name_str) = name.as_str() {
            if name_str.len() > 255 {
                return Err(anyhow!("Field 'name' too long (max 255 characters)"));
            }
        } else {
            return Err(anyhow!("Field 'name' must be a string"));
        }
    }

    if let Some(description) = obj.get("description") {
        if let Some(desc_str) = description.as_str() {
            if desc_str.len() > 1000 {
                return Err(anyhow!("Field 'description' too long (max 1000 characters)"));
            }
        } else {
            return Err(anyhow!("Field 'description' must be a string"));
        }
    }

    if let Some(version) = obj.get("version") {
        if let Some(version_str) = version.as_str() {
            if version_str.len() > 50 {
                return Err(anyhow!("Field 'version' too long (max 50 characters)"));
            }
        } else {
            return Err(anyhow!("Field 'version' must be a string"));
        }
    }

    if let Some(tags) = obj.get("tags") {
        if let Some(tags_array) = tags.as_array() {
            if tags_array.len() > 20 {
                return Err(anyhow!("Too many tags (max 20)"));
            }
            
            for tag in tags_array {
                if let Some(tag_str) = tag.as_str() {
                    if tag_str.len() > 50 {
                        return Err(anyhow!("Tag too long (max 50 characters)"));
                    }
                    if tag_str.is_empty() {
                        return Err(anyhow!("Tags cannot be empty"));
                    }
                } else {
                    return Err(anyhow!("All tags must be strings"));
                }
            }
        } else {
            return Err(anyhow!("Field 'tags' must be an array"));
        }
    }

    Ok(())
}

fn validate_dublin_core_meta(meta: &Value) -> Result<()> {
    let obj = meta.as_object()
        .ok_or_else(|| anyhow!("Metadata must be a JSON object"))?;

    // Required Dublin Core fields
    let required_fields = [
        "creation_dt", "creator", "file_name", "file_type", "file_size",
        "org_lab", "description", "data_source", "data_collection_method", "version"
    ];

    for field in &required_fields {
        if !obj.contains_key(*field) {
            return Err(anyhow!("Missing required Dublin Core field: {}", field));
        }
    }

    // Validate creation_dt format
    if let Some(creation_dt) = obj.get("creation_dt") {
        if let Some(dt_str) = creation_dt.as_str() {
            // Try to parse as ISO 8601
            chrono::DateTime::parse_from_rfc3339(dt_str)
                .map_err(|_| anyhow!("Invalid creation_dt format, must be ISO 8601"))?;
        } else {
            return Err(anyhow!("Field 'creation_dt' must be a string"));
        }
    }

    // Validate file_size
    if let Some(file_size) = obj.get("file_size") {
        if let Some(size) = file_size.as_i64() {
            if size < 0 {
                return Err(anyhow!("File size cannot be negative"));
            }
            if size > 10_000_000_000_000 { // 10TB limit
                return Err(anyhow!("File size too large (max 10TB)"));
            }
        } else {
            return Err(anyhow!("Field 'file_size' must be a number"));
        }
    }

    // Validate string fields
    let string_fields = ["creator", "file_name", "file_type", "org_lab", "description", "data_source", "data_collection_method", "version"];
    for field in &string_fields {
        if let Some(value) = obj.get(*field) {
            if let Some(str_value) = value.as_str() {
                if str_value.is_empty() {
                    return Err(anyhow!("Field '{}' cannot be empty", field));
                }
                if str_value.len() > 500 {
                    return Err(anyhow!("Field '{}' too long (max 500 characters)", field));
                }
            } else {
                return Err(anyhow!("Field '{}' must be a string", field));
            }
        }
    }

    // Validate optional fields
    if let Some(notes) = obj.get("notes") {
        if let Some(notes_str) = notes.as_str() {
            if notes_str.len() > 2000 {
                return Err(anyhow!("Field 'notes' too long (max 2000 characters)"));
            }
        } else {
            return Err(anyhow!("Field 'notes' must be a string"));
        }
    }

    if let Some(license) = obj.get("license") {
        if let Some(license_str) = license.as_str() {
            if license_str.len() > 100 {
                return Err(anyhow!("Field 'license' too long (max 100 characters)"));
            }
        } else {
            return Err(anyhow!("Field 'license' must be a string"));
        }
    }

    // Validate tags array
    if let Some(tags) = obj.get("tags") {
        if let Some(tags_array) = tags.as_array() {
            if tags_array.len() > 50 {
                return Err(anyhow!("Too many tags (max 50)"));
            }
            
            for tag in tags_array {
                if let Some(tag_str) = tag.as_str() {
                    if tag_str.is_empty() {
                        return Err(anyhow!("Tags cannot be empty"));
                    }
                    if tag_str.len() > 100 {
                        return Err(anyhow!("Tag too long (max 100 characters)"));
                    }
                } else {
                    return Err(anyhow!("All tags must be strings"));
                }
            }
        } else {
            return Err(anyhow!("Field 'tags' must be an array"));
        }
    }

    Ok(())
}

/// Content type validation
pub fn validate_content_type(content_type: &str) -> Result<()> {
    if content_type.is_empty() {
        return Err(anyhow!("Content type cannot be empty"));
    }

    // Basic MIME type validation
    let mime_regex = Regex::new(r"^[a-zA-Z0-9][a-zA-Z0-9!#$&\-\^_]*/[a-zA-Z0-9][a-zA-Z0-9!#$&\-\^_]*$")?;
    if !mime_regex.is_match(content_type) {
        return Err(anyhow!("Invalid content type format"));
    }

    // Check for dangerous content types
    let dangerous_types = [
        "application/x-executable",
        "application/x-msdownload",
        "application/x-msdos-program",
        "application/x-winexe",
        "application/x-msi",
    ];

    if dangerous_types.contains(&content_type) {
        return Err(anyhow!("Dangerous content type not allowed: {}", content_type));
    }

    Ok(())
}

/// File size validation
pub fn validate_file_size(size: u64, max_size: Option<u64>) -> Result<()> {
    let max_size = max_size.unwrap_or(10_000_000_000); // 10GB default

    if size == 0 {
        return Err(anyhow!("File size cannot be zero"));
    }

    if size > max_size {
        return Err(anyhow!(
            "File size {} exceeds maximum allowed size {}",
            size,
            max_size
        ));
    }

    Ok(())
}

/// Idempotency key validation
pub fn validate_idempotency_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(anyhow!("Idempotency key cannot be empty"));
    }

    if key.len() > 128 {
        return Err(anyhow!("Idempotency key too long (max 128 characters)"));
    }

    // Allow alphanumeric, hyphens, underscores
    let key_regex = Regex::new(r"^[a-zA-Z0-9_-]+$")?;
    if !key_regex.is_match(key) {
        return Err(anyhow!(
            "Idempotency key can only contain alphanumeric characters, hyphens, and underscores"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_repo_name() {
        // Valid names
        assert!(validate_repo_name("my-repo").is_ok());
        assert!(validate_repo_name("my_repo").is_ok());
        assert!(validate_repo_name("my.repo").is_ok());
        assert!(validate_repo_name("repo123").is_ok());

        // Invalid names
        assert!(validate_repo_name("").is_err());
        assert!(validate_repo_name("-repo").is_err());
        assert!(validate_repo_name("repo-").is_err());
        assert!(validate_repo_name(".repo").is_err());
        assert!(validate_repo_name("repo.").is_err());
        assert!(validate_repo_name("repo..name").is_err());
        assert!(validate_repo_name("repo name").is_err());
        assert!(validate_repo_name("repo@name").is_err());
    }

    #[test]
    fn test_normalize_path() {
        // Valid paths
        assert_eq!(normalize_path("path/to/file").unwrap(), "path/to/file");
        assert_eq!(normalize_path("/path/to/file/").unwrap(), "path/to/file");
        assert_eq!(normalize_path("path//to//file").unwrap(), "path/to/file");
        assert_eq!(normalize_path("path\\to\\file").unwrap(), "path/to/file");

        // Invalid paths
        assert!(normalize_path("").is_err());
        assert!(normalize_path("/").is_err());
        assert!(normalize_path("path/../file").is_err());
        assert!(normalize_path("path/..").is_err());
        assert!(normalize_path("path\0file").is_err());
    }

    #[test]
    fn test_validate_meta_v1_0() {
        // Valid metadata
        let valid_meta = json!({
            "name": "test",
            "description": "test description",
            "version": "1.0",
            "tags": ["tag1", "tag2"]
        });
        assert!(validate_meta(&valid_meta, Some("1.0")).is_ok());

        // Missing required field
        let invalid_meta = json!({
            "description": "test description"
        });
        assert!(validate_meta(&invalid_meta, Some("1.0")).is_err());

        // Empty name
        let empty_name_meta = json!({
            "name": "",
            "description": "test description"
        });
        assert!(validate_meta(&empty_name_meta, Some("1.0")).is_err());
    }

    #[test]
    fn test_validate_dublin_core_meta() {
        // Valid Dublin Core metadata
        let valid_meta = json!({
            "creation_dt": "2025-01-17T18:28:00Z",
            "creator": "test@example.com",
            "file_name": "test.csv",
            "file_type": "text/csv",
            "file_size": 1234,
            "org_lab": "TestLab",
            "description": "Test dataset",
            "data_source": "sensor",
            "data_collection_method": "manual",
            "version": "1.0"
        });
        assert!(validate_meta(&valid_meta, Some("dublin-core")).is_ok());

        // Missing required field
        let invalid_meta = json!({
            "creator": "test@example.com",
            "file_name": "test.csv"
        });
        assert!(validate_meta(&invalid_meta, Some("dublin-core")).is_err());

        // Invalid date format
        let invalid_date_meta = json!({
            "creation_dt": "invalid-date",
            "creator": "test@example.com",
            "file_name": "test.csv",
            "file_type": "text/csv",
            "file_size": 1234,
            "org_lab": "TestLab",
            "description": "Test dataset",
            "data_source": "sensor",
            "data_collection_method": "manual",
            "version": "1.0"
        });
        assert!(validate_meta(&invalid_date_meta, Some("dublin-core")).is_err());
    }

    #[test]
    fn test_validate_content_type() {
        // Valid content types
        assert!(validate_content_type("text/plain").is_ok());
        assert!(validate_content_type("application/json").is_ok());
        assert!(validate_content_type("image/png").is_ok());

        // Invalid content types
        assert!(validate_content_type("").is_err());
        assert!(validate_content_type("invalid").is_err());
        assert!(validate_content_type("application/x-executable").is_err());
    }

    #[test]
    fn test_validate_file_size() {
        // Valid sizes
        assert!(validate_file_size(1000, None).is_ok());
        assert!(validate_file_size(1000, Some(2000)).is_ok());

        // Invalid sizes
        assert!(validate_file_size(0, None).is_err());
        assert!(validate_file_size(2000, Some(1000)).is_err());
    }

    #[test]
    fn test_validate_idempotency_key() {
        // Valid keys
        assert!(validate_idempotency_key("key123").is_ok());
        assert!(validate_idempotency_key("key-123").is_ok());
        assert!(validate_idempotency_key("key_123").is_ok());

        // Invalid keys
        assert!(validate_idempotency_key("").is_err());
        assert!(validate_idempotency_key("key 123").is_err());
        assert!(validate_idempotency_key("key@123").is_err());
    }
}
