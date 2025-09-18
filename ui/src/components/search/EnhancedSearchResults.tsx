import React from 'react'
import { Link } from 'react-router-dom'
import { Card } from '@/components/ui/card'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/label'
import { 
  FileText, 
  Database, 
  Image, 
  File, 
  Download, 
  Eye,
  Calendar,
  User,
  Building,
  Tag
} from 'lucide-react'

interface SearchResult {
  id: string
  repo_id: string
  repo_name: string
  path: string
  file_name: string
  file_type: string
  file_size: number
  creation_dt: string
  creator?: string
  org_lab?: string
  tags?: string[]
  description?: string
  object_sha256: string
  meta?: Record<string, any>
}

interface SearchResultsProps {
  results: SearchResult[]
  total: number
  isLoading: boolean
  onLoadMore?: () => void
  hasMore?: boolean
}

const EnhancedSearchResults: React.FC<SearchResultsProps> = ({
  results,
  total,
  isLoading,
  onLoadMore,
  hasMore = false
}) => {
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }

  const getFileIcon = (fileType: string) => {
    if (fileType.includes('csv') || fileType.includes('spreadsheet')) {
      return <Database className="h-5 w-5 text-green-600" />
    } else if (fileType.includes('image')) {
      return <Image className="h-5 w-5 text-blue-600" />
    } else if (fileType.includes('json') || fileType.includes('xml')) {
      return <FileText className="h-5 w-5 text-orange-600" />
    } else {
      return <File className="h-5 w-5 text-gray-600" />
    }
  }

  const formatDate = (dateString: string): string => {
    try {
      return new Date(dateString).toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric'
      })
    } catch {
      return dateString
    }
  }

  if (isLoading && results.length === 0) {
    return (
      <div className="space-y-4">
        {[...Array(5)].map((_, i) => (
          <Card key={i} className="p-4 animate-pulse">
            <div className="flex items-start gap-4">
              <div className="w-8 h-8 bg-gray-200 rounded"></div>
              <div className="flex-1 space-y-2">
                <div className="h-4 bg-gray-200 rounded w-3/4"></div>
                <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                <div className="h-3 bg-gray-200 rounded w-1/4"></div>
              </div>
            </div>
          </Card>
        ))}
      </div>
    )
  }

  if (results.length === 0) {
    return (
      <Card className="p-8 text-center">
        <File className="h-12 w-12 text-gray-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 mb-2">No results found</h3>
        <p className="text-gray-500">
          Try adjusting your search terms or filters to find what you're looking for.
        </p>
      </Card>
    )
  }

  return (
    <div className="space-y-4">
      {/* Results Header */}
      <div className="flex items-center justify-between">
        <div className="text-sm text-gray-600">
          Showing {results.length} of {total.toLocaleString()} results
        </div>
        <div className="text-sm text-gray-500">
          {total > 0 && (
            <span>
              {total === 1 ? '1 result' : `${total.toLocaleString()} results`}
            </span>
          )}
        </div>
      </div>

      {/* Results List */}
      <div className="space-y-3">
        {results.map((result) => (
          <Card key={result.id} className="p-4 hover:shadow-md transition-shadow">
            <div className="flex items-start gap-4">
              {/* File Icon */}
              <div className="flex-shrink-0 mt-1">
                {getFileIcon(result.file_type)}
              </div>

              {/* Content */}
              <div className="flex-1 min-w-0">
                {/* Title and Path */}
                <div className="flex items-start justify-between gap-4">
                  <div className="min-w-0 flex-1">
                    <Link
                      to={`/repos/${result.repo_name}/blob/main${result.path}`}
                      className="text-blue-600 hover:text-blue-800 font-medium truncate block"
                    >
                      {result.file_name}
                    </Link>
                    <div className="text-sm text-gray-500 truncate">
                      {result.repo_name}{result.path}
                    </div>
                  </div>
                  
                  {/* Actions */}
                  <div className="flex items-center gap-2 flex-shrink-0">
                    <Button
                      variant="outline"
                      size="sm"
                      asChild
                    >
                      <Link to={`/repos/${result.repo_name}/blob/main${result.path}`}>
                        <Eye className="h-4 w-4 mr-1" />
                        View
                      </Link>
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      asChild
                    >
                      <Link to={`/repos/${result.repo_name}/blob/main${result.path}?download=true`}>
                        <Download className="h-4 w-4 mr-1" />
                        Download
                      </Link>
                    </Button>
                  </div>
                </div>

                {/* Description */}
                {result.description && (
                  <div className="mt-2 text-sm text-gray-600 line-clamp-2">
                    {result.description}
                  </div>
                )}

                {/* Metadata */}
                <div className="mt-3 flex flex-wrap items-center gap-4 text-xs text-gray-500">
                  {/* File Size */}
                  <div className="flex items-center gap-1">
                    <File className="h-3 w-3" />
                    {formatFileSize(result.file_size)}
                  </div>

                  {/* Creation Date */}
                  {result.creation_dt && (
                    <div className="flex items-center gap-1">
                      <Calendar className="h-3 w-3" />
                      {formatDate(result.creation_dt)}
                    </div>
                  )}

                  {/* Creator */}
                  {result.creator && (
                    <div className="flex items-center gap-1">
                      <User className="h-3 w-3" />
                      {result.creator}
                    </div>
                  )}

                  {/* Organization/Lab */}
                  {result.org_lab && (
                    <div className="flex items-center gap-1">
                      <Building className="h-3 w-3" />
                      {result.org_lab}
                    </div>
                  )}

                  {/* File Type */}
                  <Badge variant="secondary" className="text-xs">
                    {result.file_type.split('/').pop() || result.file_type}
                  </Badge>
                </div>

                {/* Tags */}
                {result.tags && result.tags.length > 0 && (
                  <div className="mt-2 flex flex-wrap gap-1">
                    {result.tags.slice(0, 5).map((tag) => (
                      <Badge
                        key={tag}
                        variant="outline"
                        className="text-xs flex items-center gap-1"
                      >
                        <Tag className="h-3 w-3" />
                        {tag}
                      </Badge>
                    ))}
                    {result.tags.length > 5 && (
                      <Badge variant="outline" className="text-xs">
                        +{result.tags.length - 5} more
                      </Badge>
                    )}
                  </div>
                )}
              </div>
            </div>
          </Card>
        ))}
      </div>

      {/* Load More Button */}
      {hasMore && onLoadMore && (
        <div className="text-center pt-4">
          <Button
            onClick={onLoadMore}
            disabled={isLoading}
            variant="outline"
          >
            {isLoading ? 'Loading...' : 'Load More Results'}
          </Button>
        </div>
      )}
    </div>
  )
}

export default EnhancedSearchResults
