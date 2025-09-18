// Mobile-optimized repository browser component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  Folder, 
  File, 
  ChevronRight, 
  MoreVertical, 
  Download, 
  Eye,
  Share,
  Star,
  Clock,
  User,
  Tag
} from 'lucide-react'
import { RepositoryEntry } from '@/types/repository'

interface MobileRepositoryBrowserProps {
  entries: RepositoryEntry[]
  currentPath: string
  onNavigate: (path: string) => void
  onEntryClick: (entry: RepositoryEntry) => void
  onEntryAction?: (entry: RepositoryEntry, action: string) => void
  className?: string
}

export const MobileRepositoryBrowser: React.FC<MobileRepositoryBrowserProps> = ({
  entries,
  currentPath,
  onNavigate,
  onEntryClick,
  onEntryAction,
  className = ''
}) => {
  const [selectedEntry, setSelectedEntry] = useState<RepositoryEntry | null>(null)
  const [showActions, setShowActions] = useState(false)

  // Group entries by type (folders first, then files)
  const sortedEntries = [...entries].sort((a, b) => {
    if (a.type === 'directory' && b.type !== 'directory') return -1
    if (a.type !== 'directory' && b.type === 'directory') return 1
    return a.name.localeCompare(b.name)
  })

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

  const getFileIcon = (entry: RepositoryEntry) => {
    if (entry.type === 'directory') {
      return <Folder className="w-5 h-5 text-blue-500" />
    }
    
    const extension = entry.name.split('.').pop()?.toLowerCase()
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

  const handleEntryPress = useCallback((entry: RepositoryEntry) => {
    if (entry.type === 'directory') {
      onNavigate(entry.path)
    } else {
      onEntryClick(entry)
    }
  }, [onNavigate, onEntryClick])

  const handleLongPress = useCallback((entry: RepositoryEntry) => {
    setSelectedEntry(entry)
    setShowActions(true)
  }, [])

  const handleAction = useCallback((action: string) => {
    if (selectedEntry && onEntryAction) {
      onEntryAction(selectedEntry, action)
    }
    setShowActions(false)
    setSelectedEntry(null)
  }, [selectedEntry, onEntryAction])

  return (
    <div className={`bg-white ${className}`}>
      {/* Breadcrumb */}
      {currentPath !== '/' && (
        <div className="px-4 py-3 border-b border-gray-200">
          <div className="flex items-center space-x-1 text-sm text-gray-600">
            <button
              onClick={() => onNavigate('/')}
              className="hover:text-gray-900 transition-colors"
            >
              Root
            </button>
            {currentPath.split('/').filter(Boolean).map((segment, index, array) => (
              <React.Fragment key={index}>
                <ChevronRight className="w-4 h-4" />
                <button
                  onClick={() => {
                    const path = '/' + array.slice(0, index + 1).join('/')
                    onNavigate(path)
                  }}
                  className="hover:text-gray-900 transition-colors"
                >
                  {segment}
                </button>
              </React.Fragment>
            ))}
          </div>
        </div>
      )}

      {/* Entry List */}
      <div className="divide-y divide-gray-100">
        {sortedEntries.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <Folder className="w-12 h-12 mx-auto mb-3 text-gray-300" />
            <p className="text-sm">This folder is empty</p>
          </div>
        ) : (
          sortedEntries.map((entry) => (
            <div
              key={entry.path}
              className="p-4 hover:bg-gray-50 transition-colors"
              onClick={() => handleEntryPress(entry)}
              onContextMenu={(e) => {
                e.preventDefault()
                handleLongPress(entry)
              }}
            >
              <div className="flex items-center space-x-3">
                {/* Icon */}
                <div className="flex-shrink-0">
                  {getFileIcon(entry)}
                </div>

                {/* Entry Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between">
                    <h3 className="text-sm font-medium text-gray-900 truncate">
                      {entry.name}
                    </h3>
                    {entry.type === 'directory' && (
                      <ChevronRight className="w-4 h-4 text-gray-400 flex-shrink-0" />
                    )}
                  </div>
                  
                  <div className="mt-1 flex items-center space-x-4 text-xs text-gray-500">
                    {entry.type !== 'directory' && entry.size && (
                      <span>{formatFileSize(entry.size)}</span>
                    )}
                    <span className="flex items-center space-x-1">
                      <Clock className="w-3 h-3" />
                      <span>{formatDate(entry.lastModified)}</span>
                    </span>
                    {entry.author && (
                      <span className="flex items-center space-x-1">
                        <User className="w-3 h-3" />
                        <span>{entry.author}</span>
                      </span>
                    )}
                  </div>

                  {/* Tags */}
                  {entry.tags && entry.tags.length > 0 && (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {entry.tags.slice(0, 3).map((tag, index) => (
                        <span
                          key={index}
                          className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800"
                        >
                          <Tag className="w-3 h-3 mr-1" />
                          {tag}
                        </span>
                      ))}
                      {entry.tags.length > 3 && (
                        <span className="text-xs text-gray-500">
                          +{entry.tags.length - 3} more
                        </span>
                      )}
                    </div>
                  )}
                </div>

                {/* Actions Button */}
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    setSelectedEntry(entry)
                    setShowActions(true)
                  }}
                  className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-md transition-colors"
                >
                  <MoreVertical className="w-4 h-4" />
                </button>
              </div>
            </div>
          ))
        )}
      </div>

      {/* Action Sheet */}
      {showActions && selectedEntry && (
        <div className="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-end">
          <div className="w-full bg-white rounded-t-lg">
            {/* Header */}
            <div className="p-4 border-b border-gray-200">
              <div className="flex items-center space-x-3">
                {getFileIcon(selectedEntry)}
                <div className="flex-1 min-w-0">
                  <h3 className="text-sm font-medium text-gray-900 truncate">
                    {selectedEntry.name}
                  </h3>
                  <p className="text-xs text-gray-500">
                    {selectedEntry.type === 'directory' ? 'Folder' : 'File'}
                  </p>
                </div>
                <button
                  onClick={() => setShowActions(false)}
                  className="p-2 text-gray-400 hover:text-gray-600 transition-colors"
                >
                  <X className="w-5 h-5" />
                </button>
              </div>
            </div>

            {/* Actions */}
            <div className="py-2">
              <button
                onClick={() => handleAction('view')}
                className="w-full flex items-center space-x-3 px-4 py-3 text-left hover:bg-gray-50 transition-colors"
              >
                <Eye className="w-5 h-5 text-gray-400" />
                <span className="text-sm text-gray-900">View</span>
              </button>
              
              <button
                onClick={() => handleAction('download')}
                className="w-full flex items-center space-x-3 px-4 py-3 text-left hover:bg-gray-50 transition-colors"
              >
                <Download className="w-5 h-5 text-gray-400" />
                <span className="text-sm text-gray-900">Download</span>
              </button>
              
              <button
                onClick={() => handleAction('share')}
                className="w-full flex items-center space-x-3 px-4 py-3 text-left hover:bg-gray-50 transition-colors"
              >
                <Share className="w-5 h-5 text-gray-400" />
                <span className="text-sm text-gray-900">Share</span>
              </button>
              
              <button
                onClick={() => handleAction('favorite')}
                className="w-full flex items-center space-x-3 px-4 py-3 text-left hover:bg-gray-50 transition-colors"
              >
                <Star className="w-5 h-5 text-gray-400" />
                <span className="text-sm text-gray-900">Add to Favorites</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

export default MobileRepositoryBrowser
