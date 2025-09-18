import React, { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { Button } from '@/components/ui/button'
import { FileDropzone } from '@/components/ui/FileDropzone'
import { JsonSchemaForm } from '@/components/ui/JsonSchemaForm'
import { ProgressBar } from '@/components/ui/ProgressBar'
import { useAppStore } from '@/app/store'

interface FileWithProgress extends File {
  progress: number
  status: 'pending' | 'uploading' | 'completed' | 'error'
  error?: string
  metadata?: Record<string, any>
}

const UploadWizard: React.FC = () => {
  const { name } = useParams<{ name: string }>()
  const navigate = useNavigate()
  const { addToast } = useAppStore()
  
  const [step, setStep] = useState(1)
  const [files, setFiles] = useState<FileWithProgress[]>([])
  const [globalMetadata, setGlobalMetadata] = useState<Record<string, any>>({})
  const [isUploading, setIsUploading] = useState(false)
  const [currentFileIndex, setCurrentFileIndex] = useState(0)

  // Default Dublin Core schema for metadata
  const metadataSchema = {
    title: "File Metadata",
    description: "Provide metadata for your files using Dublin Core standard",
    fields: [
      { name: 'creator', type: 'string' as const, required: true, label: 'Creator', description: 'Person or organization who created the file' },
      { name: 'description', type: 'string' as const, required: true, label: 'Description', description: 'Brief description of the file content' },
      { name: 'org_lab', type: 'string' as const, required: true, label: 'Organization/Lab', description: 'Originating organization or laboratory' },
      { name: 'data_source', type: 'string' as const, required: true, label: 'Data Source', description: 'Source of the data' },
      { name: 'data_collection_method', type: 'string' as const, required: false, label: 'Collection Method', description: 'Method used to collect the data' },
      { name: 'version', type: 'string' as const, required: false, label: 'Version', description: 'Version of the dataset', default: '1.0' },
      { name: 'license', type: 'string' as const, required: false, label: 'License', description: 'License under which the data is shared' },
      { name: 'tags', type: 'array' as const, required: false, label: 'Tags', description: 'Keywords or tags for categorization' },
      { name: 'notes', type: 'string' as const, required: false, label: 'Notes', description: 'Additional notes or comments' }
    ]
  }

  const handleFileSelect = (selectedFiles: File[]) => {
    const filesWithProgress: FileWithProgress[] = selectedFiles.map(file => ({
      ...file,
      progress: 0,
      status: 'pending' as const,
      metadata: { ...globalMetadata }
    }))
    setFiles(filesWithProgress)
    if (selectedFiles.length > 0) {
      setStep(2)
    }
  }

  const handleMetadataChange = (metadata: Record<string, any>) => {
    setGlobalMetadata(metadata)
    // Apply to all files
    setFiles(prev => prev.map(file => ({
      ...file,
      metadata: { ...metadata, file_name: file.name, file_size: file.size, file_type: file.type }
    })))
  }

  const simulateUpload = async (file: FileWithProgress, index: number): Promise<void> => {
    return new Promise((resolve, reject) => {
      const uploadTime = 2000 + Math.random() * 3000 // 2-5 seconds
      const interval = 50
      const steps = uploadTime / interval
      let currentStep = 0

      setFiles(prev => prev.map((f, i) => 
        i === index ? { ...f, status: 'uploading' as const, progress: 0 } : f
      ))

      const timer = setInterval(() => {
        currentStep++
        const progress = (currentStep / steps) * 100

        setFiles(prev => prev.map((f, i) => 
          i === index ? { ...f, progress: Math.min(100, progress) } : f
        ))

        if (currentStep >= steps) {
          clearInterval(timer)
          
          // Simulate random failures for demo
          if (Math.random() < 0.1) { // 10% failure rate
            setFiles(prev => prev.map((f, i) => 
              i === index ? { ...f, status: 'error' as const, error: 'Upload failed' } : f
            ))
            reject(new Error('Upload failed'))
          } else {
            setFiles(prev => prev.map((f, i) => 
              i === index ? { ...f, status: 'completed' as const, progress: 100 } : f
            ))
            resolve()
          }
        }
      }, interval)
    })
  }

  const handleUpload = async () => {
    if (files.length === 0) {
      addToast('Please select files to upload', 'error')
      return
    }

    if (!globalMetadata.creator || !globalMetadata.description || !globalMetadata.org_lab || !globalMetadata.data_source) {
      addToast('Please fill in required metadata fields', 'error')
      return
    }

    setIsUploading(true)
    setStep(3)

    try {
      // Upload files one by one
      for (let i = 0; i < files.length; i++) {
        setCurrentFileIndex(i)
        await simulateUpload(files[i], i)
      }

      const successCount = files.filter(f => f.status === 'completed').length
      const errorCount = files.filter(f => f.status === 'error').length

      if (errorCount === 0) {
        addToast(`All ${successCount} files uploaded successfully`, 'success')
      } else {
        addToast(`${successCount} files uploaded, ${errorCount} failed`, 'warning')
      }

      // Navigate back after a short delay
      setTimeout(() => {
        navigate(`/repos/${name}`)
      }, 2000)

    } catch (error) {
      addToast('Upload process failed', 'error')
    } finally {
      setIsUploading(false)
    }
  }

  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold">Upload Files to {name}</h1>
          <div className="flex items-center mt-4 space-x-4">
            <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
              step >= 1 ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'
            }`}>1</div>
            <div className={`flex-1 h-1 ${step >= 2 ? 'bg-primary' : 'bg-muted'}`} />
            <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
              step >= 2 ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'
            }`}>2</div>
            <div className={`flex-1 h-1 ${step >= 3 ? 'bg-primary' : 'bg-muted'}`} />
            <div className={`w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium ${
              step >= 3 ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'
            }`}>3</div>
          </div>
          <div className="flex justify-between text-sm text-muted-foreground mt-2">
            <span>Select Files</span>
            <span>Configure Metadata</span>
            <span>Upload</span>
          </div>
        </div>

        {step === 1 && (
          <div className="space-y-6">
            <FileDropzone onFilesSelect={handleFileSelect} />
          </div>
        )}

        {step === 2 && (
          <div className="space-y-6">
            <div className="border border-border rounded-lg p-6">
              <h2 className="text-xl font-semibold mb-4">Selected Files</h2>
              <div className="space-y-2 max-h-40 overflow-y-auto">
                {files.map((file, index) => (
                  <div key={index} className="flex items-center justify-between p-2 bg-muted rounded">
                    <span className="text-sm font-medium">{file.name}</span>
                    <span className="text-sm text-muted-foreground">{formatFileSize(file.size)}</span>
                  </div>
                ))}
              </div>
            </div>

            <div className="border border-border rounded-lg p-6">
              <JsonSchemaForm
                schema={metadataSchema}
                data={globalMetadata}
                onChange={handleMetadataChange}
              />
            </div>

            <div className="flex justify-between">
              <Button variant="outline" onClick={() => setStep(1)}>
                Back
              </Button>
              <Button 
                onClick={() => setStep(3)}
                disabled={!globalMetadata.creator || !globalMetadata.description}
              >
                Continue to Upload
              </Button>
            </div>
          </div>
        )}

        {step === 3 && (
          <div className="space-y-6">
            <div className="border border-border rounded-lg p-6">
              <h2 className="text-xl font-semibold mb-4">Upload Progress</h2>
              <div className="space-y-4">
                {files.map((file, index) => (
                  <div key={index} className="space-y-2">
                    <div className="flex items-center justify-between">
                      <span className="text-sm font-medium">{file.name}</span>
                      <span className="text-sm text-muted-foreground">
                        {file.status === 'completed' && '✅'}
                        {file.status === 'error' && '❌'}
                        {file.status === 'uploading' && '⏳'}
                        {file.status === 'pending' && '⏸️'}
                      </span>
                    </div>
                    <ProgressBar 
                      value={file.progress} 
                      variant={file.status === 'error' ? 'error' : file.status === 'completed' ? 'success' : 'default'}
                      text={file.status === 'error' ? file.error : undefined}
                    />
                  </div>
                ))}
              </div>
            </div>

            <div className="flex justify-between">
              <Button 
                variant="outline" 
                onClick={() => setStep(2)}
                disabled={isUploading}
              >
                Back
              </Button>
              <div className="space-x-2">
                <Button
                  variant="outline"
                  onClick={() => navigate(`/repos/${name}`)}
                  disabled={isUploading}
                >
                  Cancel
                </Button>
                <Button
                  onClick={handleUpload}
                  disabled={isUploading || files.length === 0}
                >
                  {isUploading ? 'Uploading...' : 'Start Upload'}
                </Button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

export default UploadWizard
