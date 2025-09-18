import React, { useState } from 'react'
import { Button } from './button'

interface CodeViewerProps {
  code: string
  language?: string
  title?: string
  className?: string
  showCopy?: boolean
  showDownload?: boolean
}

export const CodeViewer: React.FC<CodeViewerProps> = ({
  code,
  language = 'text',
  title,
  className = "",
  showCopy = true,
  showDownload = false
}) => {
  const [copied, setCopied] = useState(false)

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(code)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('Failed to copy text: ', err)
    }
  }

  const handleDownload = () => {
    const blob = new Blob([code], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `${title || 'code'}.${getFileExtension(language)}`
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  }

  const getFileExtension = (lang: string): string => {
    switch (lang.toLowerCase()) {
      case 'turtle':
      case 'ttl':
        return 'ttl'
      case 'json':
      case 'jsonld':
        return 'json'
      case 'xml':
        return 'xml'
      case 'rdf':
        return 'rdf'
      default:
        return 'txt'
    }
  }

  const getLanguageLabel = (lang: string): string => {
    switch (lang.toLowerCase()) {
      case 'turtle':
      case 'ttl':
        return 'Turtle'
      case 'jsonld':
        return 'JSON-LD'
      case 'json':
        return 'JSON'
      case 'xml':
        return 'XML'
      case 'rdf':
        return 'RDF/XML'
      default:
        return 'Text'
    }
  }

  return (
    <div className={`border border-border rounded-lg overflow-hidden ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-muted border-b border-border">
        <div className="flex items-center space-x-2">
          {title && <span className="font-medium text-sm">{title}</span>}
          <span className="text-xs text-muted-foreground px-2 py-1 bg-background rounded">
            {getLanguageLabel(language)}
          </span>
        </div>
        
        <div className="flex items-center space-x-2">
          {showCopy && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleCopy}
              className="text-xs"
            >
              {copied ? '✓ Copied' : '📋 Copy'}
            </Button>
          )}
          {showDownload && (
            <Button
              variant="ghost"
              size="sm"
              onClick={handleDownload}
              className="text-xs"
            >
              📥 Download
            </Button>
          )}
        </div>
      </div>

      {/* Code Content */}
      <div className="relative">
        <pre className="code-block max-h-96 overflow-auto text-xs leading-relaxed">
          <code className={`language-${language}`}>
            {code}
          </code>
        </pre>
        
        {/* Line numbers for better readability */}
        <div className="absolute left-0 top-0 p-4 text-xs text-muted-foreground select-none pointer-events-none">
          {code.split('\n').map((_, index) => (
            <div key={index} className="leading-relaxed">
              {index + 1}
            </div>
          ))}
        </div>
        
        {/* Code content with left padding for line numbers */}
        <div className="absolute left-8 top-0 right-0 p-4 text-xs">
          <code className={`language-${language} whitespace-pre`}>
            {code}
          </code>
        </div>
      </div>
    </div>
  )
}
