import React, { useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { useApi } from '@/hooks/useApi'
import { Button } from '@/components/ui/button'
import { useAppStore } from '@/stores/app'

interface Entry {
  path: string
  size: number
  sha256: string
  created_at: string
  meta?: any
}

interface TreeResponse {
  entries: Entry[]
  total: number
}

const RepositoryBrowser: React.FC = () => {
  const { name } = useParams<{ name: string }>()
  const [currentPath, setCurrentPath] = useState('')
  const [currentRef, setCurrentRef] = useState('main')
  const { setCurrentRepo } = useAppStore()

  React.useEffect(() => {
    if (name) {
      setCurrentRepo(name)
    }
  }, [name, setCurrentRepo])

  const { data: treeData, isLoading } = useApi<TreeResponse>(
    `/v1/repos/${name}/tree/${currentRef}${currentPath ? `?path=${currentPath}` : ''}`
  )

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const getFileIcon = (path: string) => {
    const ext = path.split('.').pop()?.toLowerCase()
    switch (ext) {
      case 'csv':
        return '📊'
      case 'json':
        return '📄'
      case 'parquet':
        return '🗃️'
      case 'onnx':
      case 'pt':
      case 'pth':
        return '🤖'
      default:
        return '📁'
    }
  }

  if (isLoading) {
    return (
      <div className="container mx-auto px-4 py-8">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
          <p>Loading repository contents...</p>
        </div>
      </div>
    )
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-3xl font-bold">{name}</h1>
          <p className="text-muted-foreground">
            {treeData?.total || 0} items
          </p>
        </div>
        
        <div className="flex space-x-2">
          <Button asChild>
            <Link to={`/repos/${name}/upload`}>
              Upload Files
            </Link>
          </Button>
        </div>
      </div>

      {/* Branch selector */}
      <div className="mb-4">
        <select
          value={currentRef}
          onChange={(e) => setCurrentRef(e.target.value)}
          className="px-3 py-2 border border-input rounded-md"
        >
          <option value="main">main</option>
          <option value="develop">develop</option>
        </select>
      </div>

      {/* Breadcrumb */}
      {currentPath && (
        <div className="mb-4">
          <nav className="flex" aria-label="Breadcrumb">
            <ol className="flex items-center space-x-2">
              <li>
                <button
                  onClick={() => setCurrentPath('')}
                  className="text-primary hover:underline"
                >
                  {name}
                </button>
              </li>
              {currentPath.split('/').map((segment, index, array) => {
                const path = array.slice(0, index + 1).join('/')
                return (
                  <li key={path} className="flex items-center">
                    <span className="mx-2">/</span>
                    {index === array.length - 1 ? (
                      <span className="text-muted-foreground">{segment}</span>
                    ) : (
                      <button
                        onClick={() => setCurrentPath(path)}
                        className="text-primary hover:underline"
                      >
                        {segment}
                      </button>
                    )}
                  </li>
                )
              })}
            </ol>
          </nav>
        </div>
      )}

      {/* File list */}
      {treeData && treeData.entries.length > 0 ? (
        <div className="border border-border rounded-lg">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-muted">
                <tr>
                  <th className="text-left p-4">Name</th>
                  <th className="text-left p-4">Size</th>
                  <th className="text-left p-4">Modified</th>
                  <th className="text-left p-4">Actions</th>
                </tr>
              </thead>
              <tbody>
                {treeData.entries.map((entry) => (
                  <tr key={entry.path} className="border-t border-border hover:bg-accent">
                    <td className="p-4">
                      <div className="flex items-center space-x-2">
                        <span>{getFileIcon(entry.path)}</span>
                        <span className="font-medium">{entry.path}</span>
                      </div>
                    </td>
                    <td className="p-4 text-muted-foreground">
                      {formatFileSize(entry.size)}
                    </td>
                    <td className="p-4 text-muted-foreground">
                      {new Date(entry.created_at).toLocaleDateString()}
                    </td>
                    <td className="p-4">
                      <div className="flex space-x-2">
                        <Button size="sm" variant="outline">
                          Download
                        </Button>
                        <Button 
                          size="sm" 
                          variant="outline"
                          asChild
                        >
                          <Link to={`/repos/${name}/entry/${currentRef}/${entry.path}`}>
                            View Details
                          </Link>
                        </Button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      ) : (
        <div className="text-center py-12">
          <p className="text-muted-foreground mb-4">
            This repository is empty. Upload some files to get started.
          </p>
          <Button asChild>
            <Link to={`/repos/${name}/upload`}>
              Upload Files
            </Link>
          </Button>
        </div>
      )}
    </div>
  )
}

export default RepositoryBrowser
