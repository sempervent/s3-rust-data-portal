import React, { useState, useCallback } from 'react'
import { Button } from './button'

interface FileWithPreview extends File {
  preview?: string
  metadata?: any
}

interface FileDropzoneProps {
  onFilesSelect: (files: FileWithPreview[]) => void
  accept?: string
  multiple?: boolean
  maxSize?: number // in bytes
  className?: string
}

export const FileDropzone: React.FC<FileDropzoneProps> = ({
  onFilesSelect,
  accept = "*/*",
  multiple = true,
  maxSize = 100 * 1024 * 1024, // 100MB default
  className = ""
}) => {
  const [isDragOver, setIsDragOver] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const validateFiles = (files: File[]): FileWithPreview[] => {
    const validFiles: FileWithPreview[] = []
    let errorMsg = ""

    for (const file of files) {
      if (file.size > maxSize) {
        errorMsg = `File ${file.name} is too large. Maximum size is ${formatFileSize(maxSize)}.`
        continue
      }
      
      // Add preview for images
      const fileWithPreview: FileWithPreview = file
      if (file.type.startsWith('image/')) {
        fileWithPreview.preview = URL.createObjectURL(file)
      }
      
      validFiles.push(fileWithPreview)
    }

    if (errorMsg) {
      setError(errorMsg)
    } else {
      setError(null)
    }

    return validFiles
  }

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragOver(false)
    
    const files = Array.from(e.dataTransfer.files)
    const validFiles = validateFiles(files)
    onFilesSelect(validFiles)
  }, [maxSize, onFilesSelect])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragOver(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    if (!e.currentTarget.contains(e.relatedTarget as Node)) {
      setIsDragOver(false)
    }
  }, [])

  const handleFileInput = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || [])
    const validFiles = validateFiles(files)
    onFilesSelect(validFiles)
  }, [maxSize, onFilesSelect])

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  return (
    <div className={`space-y-4 ${className}`}>
      <div
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          isDragOver
            ? 'border-primary bg-primary/5'
            : 'border-border hover:border-primary/50'
        }`}
      >
        <div className="space-y-4">
          <div className="text-4xl opacity-50">📁</div>
          <div>
            <p className="text-lg font-medium">
              {isDragOver ? 'Drop files here' : 'Drop files here or click to browse'}
            </p>
            <p className="text-sm text-muted-foreground mt-2">
              {multiple ? 'Multiple files supported' : 'Single file only'} • 
              Max size: {formatFileSize(maxSize)}
            </p>
          </div>
          
          <div>
            <Button variant="outline" onClick={() => document.getElementById('file-input')?.click()}>
              Choose Files
            </Button>
            <input
              id="file-input"
              type="file"
              multiple={multiple}
              accept={accept}
              onChange={handleFileInput}
              className="hidden"
            />
          </div>
        </div>
      </div>

      {error && (
        <div className="p-3 bg-destructive/10 border border-destructive/20 rounded-md">
          <p className="text-sm text-destructive">{error}</p>
        </div>
      )}
    </div>
  )
}
