import React from 'react'
import { Button } from './button'

// Simple JSON Schema form implementation
// In a real app, you'd use @rjsf/core or similar
interface Field {
  name: string
  type: 'string' | 'number' | 'boolean' | 'array' | 'date'
  required?: boolean
  label?: string
  description?: string
  options?: string[] // for select fields
  default?: any
}

interface JsonSchemaFormProps {
  schema: {
    fields: Field[]
    title?: string
    description?: string
  }
  data?: Record<string, any>
  onChange?: (data: Record<string, any>) => void
  onSubmit?: (data: Record<string, any>) => void
  className?: string
}

export const JsonSchemaForm: React.FC<JsonSchemaFormProps> = ({
  schema,
  data = {},
  onChange,
  onSubmit,
  className = ""
}) => {
  const [formData, setFormData] = React.useState(data)

  React.useEffect(() => {
    setFormData(data)
  }, [data])

  const handleFieldChange = (fieldName: string, value: any) => {
    const newData = { ...formData, [fieldName]: value }
    setFormData(newData)
    onChange?.(newData)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    onSubmit?.(formData)
  }

  const renderField = (field: Field) => {
    const value = formData[field.name] ?? field.default ?? ''

    switch (field.type) {
      case 'string':
        if (field.options) {
          // Select field
          return (
            <select
              value={value}
              onChange={(e) => handleFieldChange(field.name, e.target.value)}
              className="form-input"
              required={field.required}
            >
              <option value="">Select...</option>
              {field.options.map(option => (
                <option key={option} value={option}>{option}</option>
              ))}
            </select>
          )
        }
        
        // Text input
        return (
          <input
            type="text"
            value={value}
            onChange={(e) => handleFieldChange(field.name, e.target.value)}
            className="form-input"
            required={field.required}
            placeholder={field.description}
          />
        )

      case 'number':
        return (
          <input
            type="number"
            value={value}
            onChange={(e) => handleFieldChange(field.name, Number(e.target.value))}
            className="form-input"
            required={field.required}
            placeholder={field.description}
          />
        )

      case 'boolean':
        return (
          <input
            type="checkbox"
            checked={value}
            onChange={(e) => handleFieldChange(field.name, e.target.checked)}
            className="rounded border-input"
          />
        )

      case 'date':
        return (
          <input
            type="date"
            value={value}
            onChange={(e) => handleFieldChange(field.name, e.target.value)}
            className="form-input"
            required={field.required}
          />
        )

      case 'array':
        // Simple comma-separated array input
        return (
          <input
            type="text"
            value={Array.isArray(value) ? value.join(', ') : value}
            onChange={(e) => handleFieldChange(field.name, e.target.value.split(',').map(s => s.trim()))}
            className="form-input"
            required={field.required}
            placeholder={field.description || "Comma-separated values"}
          />
        )

      default:
        return null
    }
  }

  return (
    <form onSubmit={handleSubmit} className={`space-y-6 ${className}`}>
      {schema.title && (
        <div>
          <h3 className="text-lg font-semibold">{schema.title}</h3>
          {schema.description && (
            <p className="text-sm text-muted-foreground mt-1">{schema.description}</p>
          )}
        </div>
      )}

      <div className="space-y-4">
        {schema.fields.map((field) => (
          <div key={field.name} className="form-field">
            <label className="form-label">
              {field.label || field.name}
              {field.required && <span className="text-destructive ml-1">*</span>}
            </label>
            {renderField(field)}
            {field.description && (
              <p className="text-xs text-muted-foreground">{field.description}</p>
            )}
          </div>
        ))}
      </div>

      {onSubmit && (
        <div className="flex justify-end">
          <Button type="submit">Save</Button>
        </div>
      )}
    </form>
  )
}
