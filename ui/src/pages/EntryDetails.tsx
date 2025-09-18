import React, { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { Tabs } from '@/components/ui/Tabs'
import { CodeViewer } from '@/components/ui/CodeViewer'
import { JsonSchemaForm } from '@/components/ui/JsonSchemaForm'
import { ModelCard } from '@/components/ui/ModelCard'
import { Modal } from '@/components/ui/Modal'
import { useAppStore } from '@/app/store'
import { useApi } from '@/hooks/useApi'

interface Entry {
  path: string
  size: number
  sha256: string
  created_at: string
  updated_at: string
  file_type: string
  meta?: Record<string, any>
}

interface CommitHistoryEntry {
  commit_id: string
  author: string
  message: string
  created_at: string
}

const EntryDetails: React.FC = () => {
  const { name: repo, ref = 'main', '*': entryPath } = useParams<{ 
    name: string 
    ref: string 
    '*': string 
  }>()
  const navigate = useNavigate()
  const { addToast } = useAppStore()
  
  const [isMetadataEditing, setIsMetadataEditing] = useState(false)
  const [metadataChanges, setMetadataChanges] = useState<Record<string, any>>({})
  const [rdfFormat, setRdfFormat] = useState<'turtle' | 'jsonld'>('turtle')

  // API calls
  const { data: entry, isLoading: entryLoading } = useApi<Entry>(
    `/v1/repos/${repo}/entries/${ref}/${entryPath}`
  )

  const { data: rdfData, isLoading: rdfLoading } = useApi<{ content: string }>(
    `/v1/repos/${repo}/rdf/${ref}/${entryPath}?format=${rdfFormat}`,
    { enabled: !!entry }
  )

  const { data: history } = useApi<CommitHistoryEntry[]>(
    `/v1/repos/${repo}/history/${entryPath}`,
    { enabled: !!entry }
  )

  const { data: previewData } = useApi<any>(
    `/v1/repos/${repo}/preview/${ref}/${entryPath}`,
    { enabled: !!entry && isPreviewableType(entry?.file_type) }
  )

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const isModelFile = (fileType: string, fileName: string): boolean => {
    return fileType.includes('onnx') || 
           fileName.endsWith('.onnx') || 
           fileName.endsWith('.pt') || 
           fileName.endsWith('.pth') ||
           fileName.endsWith('.h5')
  }

  const isPreviewableType = (fileType?: string): boolean => {
    if (!fileType) return false
    return fileType.includes('json') || 
           fileType.includes('csv') || 
           fileType.includes('text') ||
           fileType.includes('parquet')
  }

  const handleDownload = () => {
    // TODO: Implement download via presigned URL
    addToast('Download functionality will be implemented', 'info')
  }

  const handleMetadataSave = () => {
    // TODO: Implement metadata update via API
    console.log('Saving metadata:', metadataChanges)
    addToast('Metadata saved successfully', 'success')
    setIsMetadataEditing(false)
  }

  const handleCopyS3Key = () => {
    if (entry) {
      navigator.clipboard.writeText(`s3://blacklake/${repo}/${entry.sha256}`)
      addToast('S3 key copied to clipboard', 'success')
    }
  }

  const handleCopyApiLink = () => {
    const apiUrl = `${window.location.origin}/v1/repos/${repo}/blob/${ref}/${entryPath}`
    navigator.clipboard.writeText(apiUrl)
    addToast('API link copied to clipboard', 'success')
  }

  if (entryLoading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="text-center">
          <div className="loading-spinner h-8 w-8 mx-auto mb-4"></div>
          <p>Loading entry details...</p>
        </div>
      </div>
    )
  }

  if (!entry) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="text-center">
          <h1 className="text-2xl font-bold mb-4">Entry Not Found</h1>
          <p className="text-muted-foreground mb-4">
            The requested entry could not be found.
          </p>
          <Button onClick={() => navigate(`/repos/${repo}`)}>
            Back to Repository
          </Button>
        </div>
      </div>
    )
  }

  const metadataSchema = {
    title: "Edit Metadata",
    description: "Update the metadata for this entry",
    fields: [
      { name: 'creator', type: 'string' as const, required: true, label: 'Creator' },
      { name: 'description', type: 'string' as const, required: true, label: 'Description' },
      { name: 'org_lab', type: 'string' as const, required: true, label: 'Organization/Lab' },
      { name: 'data_source', type: 'string' as const, required: true, label: 'Data Source' },
      { name: 'data_collection_method', type: 'string' as const, required: false, label: 'Collection Method' },
      { name: 'version', type: 'string' as const, required: false, label: 'Version' },
      { name: 'license', type: 'string' as const, required: false, label: 'License' },
      { name: 'tags', type: 'array' as const, required: false, label: 'Tags' },
      { name: 'notes', type: 'string' as const, required: false, label: 'Notes' }
    ]
  }

  const tabs = [
    {
      id: 'overview',
      label: 'Overview',
      content: (
        <div className="space-y-6">
          {/* File Info */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-4">
              <h3 className="text-lg font-semibold">File Information</h3>
              <dl className="space-y-2">
                <div className="flex justify-between">
                  <dt className="text-sm font-medium text-muted-foreground">Size:</dt>
                  <dd className="text-sm font-mono">{formatFileSize(entry.size)}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-sm font-medium text-muted-foreground">Type:</dt>
                  <dd className="text-sm font-mono">{entry.file_type}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-sm font-medium text-muted-foreground">Created:</dt>
                  <dd className="text-sm">{new Date(entry.created_at).toLocaleString()}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-sm font-medium text-muted-foreground">Updated:</dt>
                  <dd className="text-sm">{new Date(entry.updated_at).toLocaleString()}</dd>
                </div>
                <div className="flex justify-between">
                  <dt className="text-sm font-medium text-muted-foreground">SHA256:</dt>
                  <dd className="text-sm font-mono">{entry.sha256.substring(0, 16)}...</dd>
                </div>
              </dl>
            </div>

            <div className="space-y-4">
              <h3 className="text-lg font-semibold">Actions</h3>
              <div className="space-y-2">
                <Button onClick={handleDownload} className="w-full">
                  📥 Download File
                </Button>
                <Button variant="outline" onClick={handleCopyS3Key} className="w-full">
                  📋 Copy S3 Key
                </Button>
                <Button variant="outline" onClick={handleCopyApiLink} className="w-full">
                  🔗 Copy API Link
                </Button>
              </div>
            </div>
          </div>

          {/* Tags */}
          {entry.meta?.tags && (
            <div>
              <h3 className="text-lg font-semibold mb-2">Tags</h3>
              <div className="flex flex-wrap gap-2">
                {entry.meta.tags.map((tag: string, index: number) => (
                  <span
                    key={index}
                    className="px-2 py-1 bg-primary/10 text-primary text-sm rounded-full"
                  >
                    {tag}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      )
    },
    {
      id: 'metadata',
      label: 'Metadata',
      content: (
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <h3 className="text-lg font-semibold">Metadata</h3>
            <Button
              variant="outline"
              onClick={() => {
                setMetadataChanges(entry.meta || {})
                setIsMetadataEditing(true)
              }}
            >
              ✏️ Edit
            </Button>
          </div>

          {entry.meta ? (
            <div className="code-block">
              <pre>{JSON.stringify(entry.meta, null, 2)}</pre>
            </div>
          ) : (
            <p className="text-muted-foreground">No metadata available</p>
          )}
        </div>
      )
    },
    {
      id: 'rdf',
      label: 'RDF',
      content: (
        <div className="space-y-4">
          <div className="flex justify-between items-center">
            <h3 className="text-lg font-semibold">RDF Representation</h3>
            <div className="flex items-center space-x-2">
              <select
                value={rdfFormat}
                onChange={(e) => setRdfFormat(e.target.value as 'turtle' | 'jsonld')}
                className="px-3 py-1 border border-input rounded text-sm"
              >
                <option value="turtle">Turtle</option>
                <option value="jsonld">JSON-LD</option>
              </select>
            </div>
          </div>

          {rdfLoading ? (
            <div className="text-center py-8">
              <div className="loading-spinner h-6 w-6 mx-auto mb-2"></div>
              <p className="text-sm text-muted-foreground">Loading RDF data...</p>
            </div>
          ) : rdfData ? (
            <CodeViewer
              code={rdfData.content}
              language={rdfFormat}
              title={`${entry.path} (${rdfFormat.toUpperCase()})`}
              showCopy={true}
              showDownload={true}
            />
          ) : (
            <p className="text-muted-foreground">No RDF data available</p>
          )}
        </div>
      )
    },
    {
      id: 'preview',
      label: 'Preview',
      content: (
        <div className="space-y-4">
          <h3 className="text-lg font-semibold">File Preview</h3>
          
          {isModelFile(entry.file_type, entry.path) ? (
            <ModelCard
              fileName={entry.path}
              fileSize={entry.size}
              fileSha256={entry.sha256}
              modelInfo={entry.meta?.model_info}
            />
          ) : isPreviewableType(entry.file_type) && previewData ? (
            <div className="border border-border rounded-lg p-4">
              <pre className="text-sm">{JSON.stringify(previewData, null, 2)}</pre>
            </div>
          ) : (
            <p className="text-muted-foreground">
              Preview not available for this file type
            </p>
          )}
        </div>
      )
    },
    {
      id: 'history',
      label: 'History',
      content: (
        <div className="space-y-4">
          <h3 className="text-lg font-semibold">Commit History</h3>
          
          {history && history.length > 0 ? (
            <div className="space-y-3">
              {history.map((commit) => (
                <div key={commit.commit_id} className="border border-border rounded-lg p-4">
                  <div className="flex items-start justify-between">
                    <div>
                      <p className="font-medium">{commit.message}</p>
                      <p className="text-sm text-muted-foreground">
                        by {commit.author} • {new Date(commit.created_at).toLocaleString()}
                      </p>
                    </div>
                    <code className="text-xs bg-muted px-2 py-1 rounded">
                      {commit.commit_id.substring(0, 8)}
                    </code>
                  </div>
                </div>
              ))}
            </div>
          ) : (
            <p className="text-muted-foreground">No history available</p>
          )}
        </div>
      )
    }
  ]

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <nav className="text-sm text-muted-foreground mb-2">
            <span
              className="hover:text-foreground cursor-pointer"
              onClick={() => navigate(`/repos/${repo}`)}
            >
              {repo}
            </span>
            {' / '}
            <span>{entryPath}</span>
          </nav>
          <h1 className="text-3xl font-bold">{entryPath?.split('/').pop()}</h1>
        </div>

        {/* Tabs */}
        <Tabs tabs={tabs} defaultTab="overview" />

        {/* Metadata Edit Modal */}
        <Modal
          isOpen={isMetadataEditing}
          onClose={() => setIsMetadataEditing(false)}
          title="Edit Metadata"
          size="lg"
        >
          <JsonSchemaForm
            schema={metadataSchema}
            data={metadataChanges}
            onChange={setMetadataChanges}
            onSubmit={handleMetadataSave}
          />
          <div className="flex justify-end space-x-2 mt-6">
            <Button variant="outline" onClick={() => setIsMetadataEditing(false)}>
              Cancel
            </Button>
            <Button onClick={handleMetadataSave}>
              Save Changes
            </Button>
          </div>
        </Modal>
      </div>
    </div>
  )
}

export default EntryDetails
