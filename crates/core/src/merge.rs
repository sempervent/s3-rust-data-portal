use anyhow::{anyhow, Result};
use serde_json::{Map, Value};
use std::collections::HashSet;

/// Deep merge two JSON values with precedence to the new value
/// Arrays for tags are union-deduplicated
/// null values remove the key
pub fn deep_merge(old: &Value, new: &Value) -> Result<Value> {
    match (old, new) {
        // If new is null, remove the key (return null)
        (_, Value::Null) => Ok(Value::Null),
        
        // If old is null or not an object, use new
        (Value::Null, _) | (_, Value::Object(_)) if !old.is_object() => Ok(new.clone()),
        
        // If new is not an object, use new
        (_, new) if !new.is_object() => Ok(new.clone()),
        
        // Both are objects - merge recursively
        (Value::Object(old_map), Value::Object(new_map)) => {
            let mut result = old_map.clone();
            
            for (key, new_value) in new_map {
                match key.as_str() {
                    // Special handling for tags array - union and deduplicate
                    "tags" => {
                        if let (Some(old_tags), Some(new_tags)) = (
                            result.get("tags").and_then(|v| v.as_array()),
                            new_value.as_array()
                        ) {
                            let mut tag_set = HashSet::new();
                            
                            // Add old tags
                            for tag in old_tags {
                                if let Some(tag_str) = tag.as_str() {
                                    tag_set.insert(tag_str.to_string());
                                }
                            }
                            
                            // Add new tags
                            for tag in new_tags {
                                if let Some(tag_str) = tag.as_str() {
                                    tag_set.insert(tag_str.to_string());
                                }
                            }
                            
                            let merged_tags: Vec<Value> = tag_set.into_iter()
                                .map(|s| Value::String(s))
                                .collect();
                            
                            result.insert(key.clone(), Value::Array(merged_tags));
                        } else {
                            result.insert(key.clone(), new_value.clone());
                        }
                    },
                    
                    // For other fields, merge recursively if both are objects
                    _ => {
                        if let Some(old_value) = result.get(key) {
                            if old_value.is_object() && new_value.is_object() {
                                result.insert(key.clone(), deep_merge(old_value, new_value)?);
                            } else {
                                result.insert(key.clone(), new_value.clone());
                            }
                        } else {
                            result.insert(key.clone(), new_value.clone());
                        }
                    }
                }
            }
            
            // Remove keys that are null
            result.retain(|_, v| !v.is_null());
            
            Ok(Value::Object(result))
        },
        
        // Fallback - use new value
        _ => Ok(new.clone()),
    }
}

/// Merge metadata specifically for CanonicalMeta
pub fn merge_canonical_meta(old_meta: &Value, new_meta: &Value) -> Result<Value> {
    let merged = deep_merge(old_meta, new_meta)?;
    
    // Validate that the result is still a valid CanonicalMeta structure
    validate_canonical_meta(&merged)?;
    
    Ok(merged)
}

/// Validate that a JSON value conforms to CanonicalMeta structure
fn validate_canonical_meta(meta: &Value) -> Result<()> {
    if let Some(obj) = meta.as_object() {
        // Check required fields
        let required_fields = [
            "creation_dt", "creator", "file_name", "file_type", "file_size",
            "org_lab", "description", "data_source", "data_collection_method", "version"
        ];
        
        for field in &required_fields {
            if !obj.contains_key(*field) {
                return Err(anyhow!("Missing required field: {}", field));
            }
        }
        
        // Validate field types
        if let Some(file_size) = obj.get("file_size") {
            if !file_size.is_number() {
                return Err(anyhow!("file_size must be a number"));
            }
        }
        
        if let Some(tags) = obj.get("tags") {
            if !tags.is_null() && !tags.is_array() {
                return Err(anyhow!("tags must be an array or null"));
            }
        }
        
        Ok(())
    } else {
        Err(anyhow!("Metadata must be a JSON object"))
    }
}

/// Extract changed and removed keys from metadata merge
pub fn get_metadata_changes(old_meta: &Value, new_meta: &Value) -> (Vec<String>, Vec<String>) {
    let mut changed_keys = Vec::new();
    let mut removed_keys = Vec::new();
    
    if let (Some(old_obj), Some(new_obj)) = (old_meta.as_object(), new_meta.as_object()) {
        // Find changed keys
        for (key, new_value) in new_obj {
            if let Some(old_value) = old_obj.get(key) {
                if old_value != new_value {
                    changed_keys.push(key.clone());
                }
            } else {
                changed_keys.push(key.clone());
            }
        }
        
        // Find removed keys (present in old but not in new)
        for key in old_obj.keys() {
            if !new_obj.contains_key(key) {
                removed_keys.push(key.clone());
            }
        }
    }
    
    (changed_keys, removed_keys)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge_basic() {
        let old = json!({
            "name": "old_name",
            "value": 42,
            "nested": {
                "a": 1,
                "b": 2
            }
        });
        
        let new = json!({
            "name": "new_name",
            "nested": {
                "b": 3,
                "c": 4
            },
            "extra": "new_field"
        });
        
        let result = deep_merge(&old, &new).unwrap();
        
        assert_eq!(result["name"], "new_name");
        assert_eq!(result["value"], 42); // Preserved from old
        assert_eq!(result["nested"]["a"], 1); // Preserved from old
        assert_eq!(result["nested"]["b"], 3); // Updated from new
        assert_eq!(result["nested"]["c"], 4); // Added from new
        assert_eq!(result["extra"], "new_field"); // Added from new
    }

    #[test]
    fn test_deep_merge_tags_union() {
        let old = json!({
            "tags": ["tag1", "tag2", "tag3"]
        });
        
        let new = json!({
            "tags": ["tag2", "tag3", "tag4"]
        });
        
        let result = deep_merge(&old, &new).unwrap();
        
        let tags = result["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 4);
        assert!(tags.contains(&json!("tag1")));
        assert!(tags.contains(&json!("tag2")));
        assert!(tags.contains(&json!("tag3")));
        assert!(tags.contains(&json!("tag4")));
    }

    #[test]
    fn test_deep_merge_null_removal() {
        let old = json!({
            "keep": "value",
            "remove": "old_value"
        });
        
        let new = json!({
            "remove": null,
            "add": "new_value"
        });
        
        let result = deep_merge(&old, &new).unwrap();
        
        assert_eq!(result["keep"], "value");
        assert!(!result.as_object().unwrap().contains_key("remove"));
        assert_eq!(result["add"], "new_value");
    }

    #[test]
    fn test_merge_canonical_meta() {
        let old_meta = json!({
            "creation_dt": "2023-01-01T00:00:00Z",
            "creator": "old@example.com",
            "file_name": "old.txt",
            "file_type": "text/plain",
            "file_size": 100,
            "org_lab": "OldLab",
            "description": "Old description",
            "data_source": "old_source",
            "data_collection_method": "old_method",
            "version": "1.0",
            "tags": ["old_tag"]
        });
        
        let new_meta = json!({
            "creator": "new@example.com",
            "description": "New description",
            "tags": ["new_tag", "old_tag"],
            "notes": "New notes"
        });
        
        let result = merge_canonical_meta(&old_meta, &new_meta).unwrap();
        
        assert_eq!(result["creator"], "new@example.com");
        assert_eq!(result["description"], "New description");
        assert_eq!(result["file_name"], "old.txt"); // Preserved
        assert_eq!(result["notes"], "New notes"); // Added
        
        let tags = result["tags"].as_array().unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&json!("old_tag")));
        assert!(tags.contains(&json!("new_tag")));
    }

    #[test]
    fn test_get_metadata_changes() {
        let old = json!({
            "a": 1,
            "b": 2,
            "c": 3
        });
        
        let new = json!({
            "a": 1, // unchanged
            "b": 4, // changed
            "d": 5  // added
        });
        
        let (changed, removed) = get_metadata_changes(&old, &new);
        
        assert_eq!(changed, vec!["b", "d"]);
        assert_eq!(removed, vec!["c"]);
    }

    #[test]
    fn test_validate_canonical_meta() {
        let valid_meta = json!({
            "creation_dt": "2023-01-01T00:00:00Z",
            "creator": "test@example.com",
            "file_name": "test.txt",
            "file_type": "text/plain",
            "file_size": 100,
            "org_lab": "TestLab",
            "description": "Test description",
            "data_source": "test_source",
            "data_collection_method": "test_method",
            "version": "1.0"
        });
        
        assert!(validate_canonical_meta(&valid_meta).is_ok());
        
        let invalid_meta = json!({
            "creator": "test@example.com"
            // Missing required fields
        });
        
        assert!(validate_canonical_meta(&invalid_meta).is_err());
    }
}
