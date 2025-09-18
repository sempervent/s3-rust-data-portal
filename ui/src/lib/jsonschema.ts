import { RJSFSchema, UiSchema } from '@rjsf/utils';
import { apiClient } from './api';

// Canonical metadata schema interface
export interface CanonicalMetaSchema {
  name: string;
  version: string;
  description?: string;
  fields: Record<string, FieldDefinition>;
  required_fields: string[];
}

export interface FieldDefinition {
  field_type: 'String' | 'Number' | 'Boolean' | 'Array' | 'Object' | 'DateTime';
  description?: string;
  default_value?: any;
  validation?: ValidationRule;
}

export interface ValidationRule {
  min_length?: number;
  max_length?: number;
  pattern?: string;
  min_value?: number;
  max_value?: number;
  allowed_values?: any[];
}

// Convert Blacklake schema to JSON Schema format
export function convertToJsonSchema(schema: CanonicalMetaSchema): RJSFSchema {
  const properties: Record<string, any> = {};
  const required: string[] = [];

  for (const [fieldName, fieldDef] of Object.entries(schema.fields)) {
    const property: any = {
      title: fieldName.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
      description: fieldDef.description,
    };

    // Set type and format
    switch (fieldDef.field_type) {
      case 'String':
        property.type = 'string';
        if (fieldName.includes('dt') || fieldName.includes('date')) {
          property.format = 'date-time';
        }
        break;
      case 'Number':
        property.type = 'number';
        break;
      case 'Boolean':
        property.type = 'boolean';
        break;
      case 'Array':
        property.type = 'array';
        property.items = { type: 'string' };
        break;
      case 'Object':
        property.type = 'object';
        break;
      case 'DateTime':
        property.type = 'string';
        property.format = 'date-time';
        break;
    }

    // Add validation rules
    if (fieldDef.validation) {
      const validation = fieldDef.validation;
      
      if (validation.min_length !== undefined) {
        property.minLength = validation.min_length;
      }
      if (validation.max_length !== undefined) {
        property.maxLength = validation.max_length;
      }
      if (validation.pattern) {
        property.pattern = validation.pattern;
      }
      if (validation.min_value !== undefined) {
        property.minimum = validation.min_value;
      }
      if (validation.max_value !== undefined) {
        property.maximum = validation.max_value;
      }
      if (validation.allowed_values) {
        property.enum = validation.allowed_values;
      }
    }

    // Set default value
    if (fieldDef.default_value !== undefined) {
      property.default = fieldDef.default_value;
    }

    properties[fieldName] = property;

    // Add to required if it's in the required fields list
    if (schema.required_fields.includes(fieldName)) {
      required.push(fieldName);
    }
  }

  return {
    type: 'object',
    properties,
    required,
    additionalProperties: false,
  };
}

// Generate UI schema for better form layout
export function generateUiSchema(schema: CanonicalMetaSchema): UiSchema {
  const uiSchema: UiSchema = {};

  for (const [fieldName, fieldDef] of Object.entries(schema.fields)) {
    const fieldUi: any = {};

    // Set widget types for better UX
    switch (fieldDef.field_type) {
      case 'DateTime':
        fieldUi['ui:widget'] = 'datetime';
        break;
      case 'Array':
        fieldUi['ui:widget'] = 'tags';
        fieldUi['ui:options'] = {
          placeholder: 'Add tags...',
        };
        break;
      case 'String':
        if (fieldName.includes('description') || fieldName.includes('notes')) {
          fieldUi['ui:widget'] = 'textarea';
          fieldUi['ui:options'] = {
            rows: 3,
          };
        } else if (fieldName.includes('email') || fieldName.includes('creator')) {
          fieldUi['ui:widget'] = 'email';
        }
        break;
    }

    // Set field order and grouping
    if (fieldName.includes('creation') || fieldName.includes('creator')) {
      fieldUi['ui:group'] = 'Basic Info';
    } else if (fieldName.includes('file_') || fieldName.includes('size')) {
      fieldUi['ui:group'] = 'File Info';
    } else if (fieldName.includes('org') || fieldName.includes('lab') || fieldName.includes('source')) {
      fieldUi['ui:group'] = 'Organization';
    } else if (fieldName.includes('tag') || fieldName.includes('license')) {
      fieldUi['ui:group'] = 'Classification';
    }

    uiSchema[fieldName] = fieldUi;
  }

  return uiSchema;
}

// Fetch schema from API
export async function fetchSchema(schemaName = 'default'): Promise<CanonicalMetaSchema> {
  try {
    const schema = await apiClient.getSchema(schemaName);
    return schema;
  } catch (error) {
    console.error('Failed to fetch schema:', error);
    // Return default schema as fallback
    return getDefaultSchema();
  }
}

// Default schema fallback
export function getDefaultSchema(): CanonicalMetaSchema {
  return {
    name: 'default',
    version: '1.0',
    description: 'Dublin Core metadata schema for data artifacts',
    required_fields: [
      'creation_dt',
      'creator',
      'file_name',
      'file_type',
      'file_size',
      'org_lab',
      'description',
      'data_source',
      'data_collection_method',
      'version',
    ],
    fields: {
      creation_dt: {
        field_type: 'DateTime',
        description: 'Date and time when the resource was created',
        validation: {
          pattern: '^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}Z$',
        },
      },
      creator: {
        field_type: 'String',
        description: 'Entity primarily responsible for making the resource',
        validation: {
          min_length: 1,
          max_length: 255,
          pattern: '^[^@]+@[^@]+\\.[^@]+$',
        },
      },
      file_name: {
        field_type: 'String',
        description: 'Name of the file',
        validation: {
          min_length: 1,
          max_length: 255,
        },
      },
      file_type: {
        field_type: 'String',
        description: 'MIME type of the file',
        default_value: 'application/octet-stream',
        validation: {
          min_length: 1,
          max_length: 100,
          pattern: '^[a-zA-Z0-9][a-zA-Z0-9!#$&\\-\\^_]*/[a-zA-Z0-9][a-zA-Z0-9!#$&\\-\\^_]*$',
        },
      },
      file_size: {
        field_type: 'Number',
        description: 'Size of the file in bytes',
        validation: {
          min_value: 0,
          max_value: 10000000000,
        },
      },
      org_lab: {
        field_type: 'String',
        description: 'Organization or laboratory responsible for the resource',
        validation: {
          min_length: 1,
          max_length: 100,
        },
      },
      description: {
        field_type: 'String',
        description: 'Description of the resource content',
        validation: {
          min_length: 1,
          max_length: 1000,
        },
      },
      data_source: {
        field_type: 'String',
        description: 'Source of the data',
        validation: {
          min_length: 1,
          max_length: 100,
        },
      },
      data_collection_method: {
        field_type: 'String',
        description: 'Method used to collect the data',
        validation: {
          min_length: 1,
          max_length: 100,
        },
      },
      version: {
        field_type: 'String',
        description: 'Version of the resource',
        default_value: '1.0',
        validation: {
          min_length: 1,
          max_length: 20,
          pattern: '^\\d+\\.\\d+(\\.\\d+)?$',
        },
      },
      notes: {
        field_type: 'String',
        description: 'Additional notes about the resource',
        validation: {
          max_length: 2000,
        },
      },
      tags: {
        field_type: 'Array',
        description: 'Tags for categorizing the resource',
        validation: {
          max_length: 50,
        },
      },
      license: {
        field_type: 'String',
        description: 'License for the resource',
        validation: {
          max_length: 100,
        },
      },
    },
  };
}

// Validate metadata against schema
export function validateMetadata(metadata: any, schema: CanonicalMetaSchema): string[] {
  const errors: string[] = [];

  // Check required fields
  for (const fieldName of schema.required_fields) {
    if (!(fieldName in metadata) || metadata[fieldName] === null || metadata[fieldName] === undefined) {
      errors.push(`Missing required field: ${fieldName}`);
    }
  }

  // Validate field types and constraints
  for (const [fieldName, fieldDef] of Object.entries(schema.fields)) {
    const value = metadata[fieldName];
    
    if (value === null || value === undefined) {
      continue; // Skip validation for null/undefined values
    }

    // Type validation
    switch (fieldDef.field_type) {
      case 'String':
        if (typeof value !== 'string') {
          errors.push(`${fieldName} must be a string`);
        }
        break;
      case 'Number':
        if (typeof value !== 'number') {
          errors.push(`${fieldName} must be a number`);
        }
        break;
      case 'Boolean':
        if (typeof value !== 'boolean') {
          errors.push(`${fieldName} must be a boolean`);
        }
        break;
      case 'Array':
        if (!Array.isArray(value)) {
          errors.push(`${fieldName} must be an array`);
        }
        break;
      case 'Object':
        if (typeof value !== 'object' || Array.isArray(value)) {
          errors.push(`${fieldName} must be an object`);
        }
        break;
      case 'DateTime':
        if (typeof value !== 'string') {
          errors.push(`${fieldName} must be a string`);
        }
        break;
    }

    // Validation rules
    if (fieldDef.validation) {
      const validation = fieldDef.validation;
      
      if (typeof value === 'string') {
        if (validation.min_length && value.length < validation.min_length) {
          errors.push(`${fieldName} must be at least ${validation.min_length} characters long`);
        }
        if (validation.max_length && value.length > validation.max_length) {
          errors.push(`${fieldName} must be at most ${validation.max_length} characters long`);
        }
        if (validation.pattern && !new RegExp(validation.pattern).test(value)) {
          errors.push(`${fieldName} does not match required pattern`);
        }
      }
      
      if (typeof value === 'number') {
        if (validation.min_value !== undefined && value < validation.min_value) {
          errors.push(`${fieldName} must be at least ${validation.min_value}`);
        }
        if (validation.max_value !== undefined && value > validation.max_value) {
          errors.push(`${fieldName} must be at most ${validation.max_value}`);
        }
      }
      
      if (Array.isArray(value)) {
        if (validation.max_length && value.length > validation.max_length) {
          errors.push(`${fieldName} must have at most ${validation.max_length} items`);
        }
      }
      
      if (validation.allowed_values && !validation.allowed_values.includes(value)) {
        errors.push(`${fieldName} has invalid value`);
      }
    }
  }

  return errors;
}

// Generate default metadata from file info
export function generateDefaultMetadata(file: File, userEmail?: string): any {
  const now = new Date().toISOString();
  
  return {
    creation_dt: now,
    creator: userEmail || 'unknown@example.com',
    file_name: file.name,
    file_type: file.type || 'application/octet-stream',
    file_size: file.size,
    org_lab: 'Unknown',
    description: `Uploaded file: ${file.name}`,
    data_source: 'upload',
    data_collection_method: 'manual',
    version: '1.0',
  };
}
