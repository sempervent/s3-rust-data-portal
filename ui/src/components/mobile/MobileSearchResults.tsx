// Mobile-optimized search results component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  File, 
  Folder, 
  Download, 
  Eye, 
  Share, 
  Star, 
  Clock, 
  User, 
  Tag,
  Filter,
  SortAsc,
  Grid,
  List,
  ChevronDown
} from 'lucide-react'
import { SearchResult } from '@/types/search'

interface MobileSearchResultsProps {
  results: SearchResult[]
  totalCount: number
  loading?: boolean
  onResultClick: (result: SearchResult) => void
  onLoadMore?: () => void
  className?: string
}

export const MobileSearchResults: React.FC<MobileSearchResultsProps> = ({
  results,
  totalCount,
  loading = false,
  onResultClick,
  onLoadMore,
  className = ''
}) => {
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list')
  const [showFilters, setShowFilters] = useState(false)
  const [showSort, setShowSort] = useState(false)

  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  const formatDate = (date: string) => {
    const d = new Date(date)
    const now = new Date()
    const diffTime = Math.abs(now.getTime() - d.getTime())
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))
    
    if (diffDays === 1) return 'Yesterday'
    if (diffDays < 7) return `${diffDays} days ago`
    if (diffDays < 30) return `${Math.ceil(diffDays / 7)} weeks ago`
    return d.toLocaleDateString()
  }

  const getFileIcon = (result: SearchResult) => {
    if (result.type === 'directory') {
      return <Folder className="w-5 h-5 text-blue-500" />
    }
    
    const extension = result.name.split('.').pop()?.toLowerCase()
    switch (extension) {
      case 'csv':
        return <File className="w-5 h-5 text-green-500" />
      case 'json':
        return <File className="w-5 h-5 text-yellow-500" />
      case 'pdf':
        return <File className="w-5 h-5 text-red-500" />
      case 'txt':
        return <File className="w-5 h-5 text-gray-500" />
      case 'jpg':
      case 'jpeg':
      case 'png':
      case 'gif':
        return <File className="w-5 h-5 text-purple-500" />
      default:
        return <File className="w-5 h-5 text-gray-400" />
    }
  }

  const handleResultPress = useCallback((result: SearchResult) => {
    onResultClick(result)
  }, [onResultClick])

  const handleLoadMore = useCallback(() => {
    if (onLoadMore && !loading) {
      onLoadMore()
    }
  }, [onLoadMore, loading])

  if (loading && results.length === 0) {
    return (
      <div className={`p-4 ${className}`}>
        <div className="space-y-4">
          {[...Array(3)].map((_, index) => (
            <div key={index} className="animate-pulse">
              <div className="flex items-center space-x-3">
                <div className="w-10 h-10 bg-gray-200 rounded"></div>
                <div className="flex-1 space-y-2">
                  <div className="h-4 bg-gray-200 rounded w-3/4"></div>
                  <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div className={`bg-white ${className}`}>
      {/* Results Header */}
      <div className="px-4 py-3 border-b border-gray-200">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <span className="text-sm font-medium text-gray-900">
              {totalCount.toLocaleString()} results
            </span>
            {loading && (
              <div className="w-4 h-4 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
            )}
          </div>
          
          <div className="flex items-center space-x-2">
            {/* View Mode Toggle */}
            <div className="flex items-center bg-gray-100 rounded-md p-1">
              <button
                onClick={() => setViewMode('list')}
                className={`p-1 rounded ${viewMode === 'list' ? 'bg-white shadow-sm' : ''}`}
              >
                <List className="w-4 h-4" />
              </button>
              <button
                onClick={() => setViewMode('grid')}
                className={`p-1 rounded ${viewMode === 'grid' ? 'bg-white shadow-sm' : ''}`}
              >
                <Grid className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Results List */}
      {results.length === 0 ? (
        <div className="p-8 text-center text-gray-500">
          <File className="w-12 h-12 mx-auto mb-3 text-gray-300" />
          <p className="text-sm">No results found</p>
          <p className="text-xs text-gray-400 mt-1">
            Try adjusting your search terms or filters
          </p>
        </div>
      ) : (
        <div className={viewMode === 'grid' ? 'grid grid-cols-2 gap-4 p-4' : 'divide-y divide-gray-100'}>
          {results.map((result) => (
            <div
              key={result.id}
              className={`${
                viewMode === 'list' 
                  ? 'p-4 hover:bg-gray-50 transition-colors' 
                  : 'p-3 bg-white rounded-lg border border-gray-200 hover:shadow-md transition-shadow'
              }`}
              onClick={() => handleResultPress(result)}
            >
              <div className="flex items-start space-x-3">
                {/* Icon */}
                <div className="flex-shrink-0">
                  {getFileIcon(result)}
                </div>

                {/* Content */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-start justify-between">
                    <div className="flex-1 min-w-0">
                      <h3 className="text-sm font-medium text-gray-900 truncate">
                        {result.name}
                      </h3>
                      <p className="text-xs text-gray-600 mt-1 line-clamp-2">
                        {result.description || result.path}
                      </p>
                    </div>
                    
                    {viewMode === 'list' && (
                      <button className="p-1 text-gray-400 hover:text-gray-600 transition-colors">
                        <MoreVertical className="w-4 h-4" />
                      </button>
                    )}
                  </div>
                  
                  {/* Metadata */}
                  <div className="mt-2 flex items-center space-x-4 text-xs text-gray-500">
                    {result.size && (
                      <span>{formatFileSize(result.size)}</span>
                    )}
                    <span className="flex items-center space-x-1">
                      <Clock className="w-3 h-3" />
                      <span>{formatDate(result.lastModified)}</span>
                    </span>
                    {result.author && (
                      <span className="flex items-center space-x-1">
                        <User className="w-3 h-3" />
                        <span>{result.author}</span>
                      </span>
                    )}
                  </div>

                  {/* Tags */}
                  {result.tags && result.tags.length > 0 && (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {result.tags.slice(0, 3).map((tag, index) => (
                        <span
                          key={index}
                          className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800"
                        >
                          <Tag className="w-3 h-3 mr-1" />
                          {tag}
                        </span>
                      ))}
                      {result.tags.length > 3 && (
                        <span className="text-xs text-gray-500">
                          +{result.tags.length - 3} more
                        </span>
                      )}
                    </div>
                  )}

                  {/* Repository */}
                  {result.repository && (
                    <div className="mt-2 text-xs text-gray-500">
                      <span className="font-medium">Repository:</span> {result.repository}
                    </div>
                  )}
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Load More Button */}
      {results.length > 0 && results.length < totalCount && (
        <div className="p-4 border-t border-gray-200">
          <button
            onClick={handleLoadMore}
            disabled={loading}
            className="w-full py-3 px-4 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {loading ? (
              <div className="flex items-center justify-center space-x-2">
                <div className="w-4 h-4 border-2 border-gray-500 border-t-transparent rounded-full animate-spin"></div>
                <span>Loading more...</span>
              </div>
            ) : (
              `Load more (${totalCount - results.length} remaining)`
            )}
          </button>
        </div>
      )}
    </div>
  )
}

export default MobileSearchResults
