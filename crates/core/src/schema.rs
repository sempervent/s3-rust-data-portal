use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Metadata schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataSchema {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub fields: HashMap<String, FieldDefinition>,
    pub required_fields: Vec<String>,
}

/// Field definition in a metadata schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub field_type: FieldType,
    pub description: Option<String>,
    pub default_value: Option<Value>,
    pub validation: Option<ValidationRule>,
}

/// Field types supported in metadata schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    DateTime,
}

/// Validation rules for fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub allowed_values: Option<Vec<Value>>,
}

/// Schema registry for managing metadata schemas
#[derive(Debug, Clone)]
pub struct SchemaRegistry {
    schemas: HashMap<String, MetadataSchema>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
        }
    }

    pub fn register_schema(&mut self, schema: MetadataSchema) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    pub fn get_schema(&self, name: &str) -> Option<&MetadataSchema> {
        self.schemas.get(name)
    }

    pub fn get_default_schema(&self) -> Option<&MetadataSchema> {
        self.schemas.get("default")
    }

    pub fn list_schemas(&self) -> Vec<&MetadataSchema> {
        self.schemas.values().collect()
    }
}

impl Default for SchemaRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Register the default Dublin Core schema
        let default_schema = create_dublin_core_schema();
        registry.register_schema(default_schema);
        
        registry
    }
}

/// Create the default Dublin Core metadata schema
pub fn create_dublin_core_schema() -> MetadataSchema {
    let mut fields = HashMap::new();
    
    // Required fields
    fields.insert("creation_dt".to_string(), FieldDefinition {
        field_type: FieldType::DateTime,
        description: Some("Date and time when the resource was created".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: None,
            max_length: None,
            pattern: Some(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$".to_string()),
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("creator".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("Entity primarily responsible for making the resource".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: Some(1),
            max_length: Some(255),
            pattern: Some(r"^[^@]+@[^@]+\.[^@]+$".to_string()), // Email pattern
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("file_name".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("Name of the file".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: Some(1),
            max_length: Some(255),
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("file_type".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("MIME type of the file".to_string()),
        default_value: Some(Value::String("application/octet-stream".to_string())),
        validation: Some(ValidationRule {
            min_length: Some(1),
            max_length: Some(100),
            pattern: Some(r"^[a-zA-Z0-9][a-zA-Z0-9!#$&\-\^_]*/[a-zA-Z0-9][a-zA-Z0-9!#$&\-\^_]*$".to_string()),
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("file_size".to_string(), FieldDefinition {
        field_type: FieldType::Number,
        description: Some("Size of the file in bytes".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: None,
            max_length: None,
            pattern: None,
            min_value: Some(0.0),
            max_value: Some(10_000_000_000.0), // 10GB max
            allowed_values: None,
        }),
    });
    
    fields.insert("org_lab".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("Organization or laboratory responsible for the resource".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: Some(1),
            max_length: Some(100),
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("description".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("Description of the resource content".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: Some(1),
            max_length: Some(1000),
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("data_source".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("Source of the data".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: Some(1),
            max_length: Some(100),
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("data_collection_method".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("Method used to collect the data".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: Some(1),
            max_length: Some(100),
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("version".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("Version of the resource".to_string()),
        default_value: Some(Value::String("1.0".to_string())),
        validation: Some(ValidationRule {
            min_length: Some(1),
            max_length: Some(20),
            pattern: Some(r"^\d+\.\d+(\.\d+)?$".to_string()), // Semantic versioning
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    // Optional fields
    fields.insert("notes".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("Additional notes about the resource".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: None,
            max_length: Some(2000),
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("tags".to_string(), FieldDefinition {
        field_type: FieldType::Array,
        description: Some("Tags for categorizing the resource".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: None,
            max_length: Some(50), // Max 50 tags
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    fields.insert("license".to_string(), FieldDefinition {
        field_type: FieldType::String,
        description: Some("License for the resource".to_string()),
        default_value: None,
        validation: Some(ValidationRule {
            min_length: None,
            max_length: Some(100),
            pattern: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
        }),
    });
    
    MetadataSchema {
        name: "default".to_string(),
        version: "1.0".to_string(),
        description: Some("Dublin Core metadata schema for data artifacts".to_string()),
        fields,
        required_fields: vec![
            "creation_dt".to_string(),
            "creator".to_string(),
            "file_name".to_string(),
            "file_type".to_string(),
            "file_size".to_string(),
            "org_lab".to_string(),
            "description".to_string(),
            "data_source".to_string(),
            "data_collection_method".to_string(),
            "version".to_string(),
        ],
    }
}

/// Validate metadata against a schema
pub fn validate_metadata(metadata: &Value, schema: &MetadataSchema) -> Result<()> {
    if let Some(obj) = metadata.as_object() {
        // Check required fields
        for field_name in &schema.required_fields {
            if !obj.contains_key(field_name) {
                return Err(anyhow!("Missing required field: {}", field_name));
            }
        }
        
        // Validate each field
        for (field_name, field_value) in obj {
            if let Some(field_def) = schema.fields.get(field_name) {
                validate_field(field_value, field_def, field_name)?;
            }
        }
        
        Ok(())
    } else {
        Err(anyhow!("Metadata must be a JSON object"))
    }
}

/// Validate a single field against its definition
fn validate_field(value: &Value, field_def: &FieldDefinition, field_name: &str) -> Result<()> {
    // Check field type
    match field_def.field_type {
        FieldType::String => {
            if !value.is_string() {
                return Err(anyhow!("Field '{}' must be a string", field_name));
            }
        },
        FieldType::Number => {
            if !value.is_number() {
                return Err(anyhow!("Field '{}' must be a number", field_name));
            }
        },
        FieldType::Boolean => {
            if !value.is_boolean() {
                return Err(anyhow!("Field '{}' must be a boolean", field_name));
            }
        },
        FieldType::Array => {
            if !value.is_array() {
                return Err(anyhow!("Field '{}' must be an array", field_name));
            }
        },
        FieldType::Object => {
            if !value.is_object() {
                return Err(anyhow!("Field '{}' must be an object", field_name));
            }
        },
        FieldType::DateTime => {
            if !value.is_string() {
                return Err(anyhow!("Field '{}' must be a string", field_name));
            }
        },
    }
    
    // Apply validation rules
    if let Some(validation) = &field_def.validation {
        apply_validation_rules(value, validation, field_name)?;
    }
    
    Ok(())
}

/// Apply validation rules to a field value
fn apply_validation_rules(value: &Value, rules: &ValidationRule, field_name: &str) -> Result<()> {
    if let Some(str_value) = value.as_str() {
        // String validations
        if let Some(min_len) = rules.min_length {
            if str_value.len() < min_len {
                return Err(anyhow!("Field '{}' must be at least {} characters long", field_name, min_len));
            }
        }
        
        if let Some(max_len) = rules.max_length {
            if str_value.len() > max_len {
                return Err(anyhow!("Field '{}' must be at most {} characters long", field_name, max_len));
            }
        }
        
        if let Some(pattern) = &rules.pattern {
            let regex = regex::Regex::new(pattern)
                .map_err(|e| anyhow!("Invalid regex pattern for field '{}': {}", field_name, e))?;
            if !regex.is_match(str_value) {
                return Err(anyhow!("Field '{}' does not match required pattern", field_name));
            }
        }
    }
    
    if let Some(num_value) = value.as_f64() {
        // Number validations
        if let Some(min_val) = rules.min_value {
            if num_value < min_val {
                return Err(anyhow!("Field '{}' must be at least {}", field_name, min_val));
            }
        }
        
        if let Some(max_val) = rules.max_value {
            if num_value > max_val {
                return Err(anyhow!("Field '{}' must be at most {}", field_name, max_val));
            }
        }
    }
    
    if let Some(array_value) = value.as_array() {
        // Array validations
        if let Some(max_len) = rules.max_length {
            if array_value.len() > max_len {
                return Err(anyhow!("Field '{}' must have at most {} items", field_name, max_len));
            }
        }
    }
    
    if let Some(allowed_values) = &rules.allowed_values {
        if !allowed_values.contains(value) {
            return Err(anyhow!("Field '{}' has invalid value", field_name));
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_schema_registry() {
        let mut registry = SchemaRegistry::new();
        let schema = create_dublin_core_schema();
        registry.register_schema(schema);
        
        assert!(registry.get_schema("default").is_some());
        assert!(registry.get_schema("nonexistent").is_none());
    }

    #[test]
    fn test_validate_metadata() {
        let schema = create_dublin_core_schema();
        
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
        
        assert!(validate_metadata(&valid_meta, &schema).is_ok());
        
        let invalid_meta = json!({
            "creator": "test@example.com"
            // Missing required fields
        });
        
        assert!(validate_metadata(&invalid_meta, &schema).is_err());
    }

    #[test]
    fn test_validate_field_types() {
        let schema = create_dublin_core_schema();
        
        let invalid_type_meta = json!({
            "creation_dt": "2023-01-01T00:00:00Z",
            "creator": "test@example.com",
            "file_name": "test.txt",
            "file_type": "text/plain",
            "file_size": "not_a_number", // Should be number
            "org_lab": "TestLab",
            "description": "Test description",
            "data_source": "test_source",
            "data_collection_method": "test_method",
            "version": "1.0"
        });
        
        assert!(validate_metadata(&invalid_type_meta, &schema).is_err());
    }
}
