import React from 'react'
import { Button } from './button'

interface ModelInfo {
  framework?: string
  opset?: string
  input_shapes?: Array<{ name: string; shape: number[]; dtype: string }>
  output_shapes?: Array<{ name: string; shape: number[]; dtype: string }>
  model_size?: number
  parameters?: number
  description?: string
}

interface ModelCardProps {
  fileName: string
  fileSize: number
  fileSha256: string
  modelInfo?: ModelInfo
  className?: string
}

export const ModelCard: React.FC<ModelCardProps> = ({
  fileName,
  fileSize,
  fileSha256,
  modelInfo,
  className = ""
}) => {
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const formatNumber = (num: number): string => {
    if (num >= 1e9) return (num / 1e9).toFixed(1) + 'B'
    if (num >= 1e6) return (num / 1e6).toFixed(1) + 'M'
    if (num >= 1e3) return (num / 1e3).toFixed(1) + 'K'
    return num.toString()
  }

  const getModelIcon = (framework?: string): string => {
    switch (framework?.toLowerCase()) {
      case 'onnx':
        return '🔮'
      case 'pytorch':
      case 'torch':
        return '🔥'
      case 'tensorflow':
        return '🧠'
      case 'keras':
        return '🎯'
      default:
        return '🤖'
    }
  }

  const handleValidateModel = () => {
    // TODO: Implement model validation endpoint call
    console.log('Validating model:', fileName)
  }

  return (
    <div className={`border border-border rounded-lg p-6 space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="flex items-center space-x-3">
          <div className="text-4xl">{getModelIcon(modelInfo?.framework)}</div>
          <div>
            <h3 className="text-lg font-semibold">{fileName}</h3>
            <p className="text-sm text-muted-foreground">
              {modelInfo?.framework || 'Unknown Framework'} Model
            </p>
          </div>
        </div>
        
        <Button variant="outline" size="sm" onClick={handleValidateModel}>
          🔍 Validate
        </Button>
      </div>

      {/* Basic Info */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="space-y-1">
          <p className="text-xs font-medium text-muted-foreground">Framework</p>
          <p className="text-sm font-mono">{modelInfo?.framework || 'Unknown'}</p>
        </div>
        
        {modelInfo?.opset && (
          <div className="space-y-1">
            <p className="text-xs font-medium text-muted-foreground">Opset</p>
            <p className="text-sm font-mono">{modelInfo.opset}</p>
          </div>
        )}
        
        <div className="space-y-1">
          <p className="text-xs font-medium text-muted-foreground">File Size</p>
          <p className="text-sm font-mono">{formatFileSize(fileSize)}</p>
        </div>
        
        {modelInfo?.parameters && (
          <div className="space-y-1">
            <p className="text-xs font-medium text-muted-foreground">Parameters</p>
            <p className="text-sm font-mono">{formatNumber(modelInfo.parameters)}</p>
          </div>
        )}
      </div>

      {/* Input/Output Shapes */}
      {(modelInfo?.input_shapes || modelInfo?.output_shapes) && (
        <div className="space-y-4">
          {modelInfo?.input_shapes && (
            <div>
              <h4 className="text-sm font-medium mb-2">Input Tensors</h4>
              <div className="space-y-2">
                {modelInfo.input_shapes.map((input, index) => (
                  <div key={index} className="flex items-center justify-between p-2 bg-muted rounded text-sm">
                    <span className="font-mono">{input.name}</span>
                    <span className="text-muted-foreground">
                      {input.shape.join(' × ')} ({input.dtype})
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
          
          {modelInfo?.output_shapes && (
            <div>
              <h4 className="text-sm font-medium mb-2">Output Tensors</h4>
              <div className="space-y-2">
                {modelInfo.output_shapes.map((output, index) => (
                  <div key={index} className="flex items-center justify-between p-2 bg-muted rounded text-sm">
                    <span className="font-mono">{output.name}</span>
                    <span className="text-muted-foreground">
                      {output.shape.join(' × ')} ({output.dtype})
                    </span>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Description */}
      {modelInfo?.description && (
        <div>
          <h4 className="text-sm font-medium mb-2">Description</h4>
          <p className="text-sm text-muted-foreground">{modelInfo.description}</p>
        </div>
      )}

      {/* Footer */}
      <div className="border-t border-border pt-4">
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span>SHA256: {fileSha256.substring(0, 16)}...</span>
          <Button variant="ghost" size="sm" className="text-xs">
            📋 Copy Hash
          </Button>
        </div>
      </div>
    </div>
  )
}
