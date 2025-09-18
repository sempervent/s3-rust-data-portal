// Mobile-optimized search history component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback } from 'react'
import { 
  Clock, 
  Trash2, 
  Search, 
  X, 
  Calendar,
  Filter,
  SortAsc
} from 'lucide-react'

interface SearchHistoryItem {
  id: string
  query: string
  timestamp: string
  resultCount: number
  filters?: string[]
  sort?: string
}

interface MobileSearchHistoryProps {
  history: SearchHistoryItem[]
  onSearch: (query: string) => void
  onClearHistory: () => void
  onRemoveItem: (id: string) => void
  className?: string
}

export const MobileSearchHistory: React.FC<MobileSearchHistoryProps> = ({
  history,
  onSearch,
  onClearHistory,
  onRemoveItem,
  className = ''
}) => {
  const [showFilters, setShowFilters] = useState(false)
  const [sortBy, setSortBy] = useState<'recent' | 'popular'>('recent')

  // Handle search from history
  const handleSearch = useCallback((item: SearchHistoryItem) => {
    onSearch(item.query)
  }, [onSearch])

  // Handle remove item
  const handleRemoveItem = useCallback((id: string, e: React.MouseEvent) => {
    e.stopPropagation()
    onRemoveItem(id)
  }, [onRemoveItem])

  // Format date
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

  // Format time
  const formatTime = (date: string) => {
    const d = new Date(date)
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  // Sort history
  const sortedHistory = [...history].sort((a, b) => {
    if (sortBy === 'recent') {
      return new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    } else {
      return b.resultCount - a.resultCount
    }
  })

  if (history.length === 0) {
    return (
      <div className={`text-center py-8 ${className}`}>
        <Clock className="w-12 h-12 mx-auto mb-3 text-gray-300" />
        <p className="text-sm text-gray-500">No search history</p>
        <p className="text-xs text-gray-400 mt-1">
          Your recent searches will appear here
        </p>
      </div>
    )
  }

  return (
    <div className={`bg-white ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200">
        <div className="flex items-center space-x-2">
          <Clock className="w-5 h-5 text-gray-600" />
          <span className="text-sm font-medium text-gray-900">
            Search History
          </span>
          <span className="text-xs text-gray-500">
            ({history.length})
          </span>
        </div>
        
        <div className="flex items-center space-x-2">
          {/* Sort Toggle */}
          <button
            onClick={() => setSortBy(sortBy === 'recent' ? 'popular' : 'recent')}
            className="flex items-center space-x-1 px-2 py-1 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded transition-colors"
          >
            <SortAsc className="w-3 h-3" />
            <span>{sortBy === 'recent' ? 'Recent' : 'Popular'}</span>
          </button>
          
          {/* Clear All */}
          <button
            onClick={onClearHistory}
            className="text-xs text-red-600 hover:text-red-700 transition-colors"
          >
            Clear All
          </button>
        </div>
      </div>

      {/* History List */}
      <div className="divide-y divide-gray-100">
        {sortedHistory.map((item) => (
          <div
            key={item.id}
            className="p-4 hover:bg-gray-50 transition-colors"
          >
            <div className="flex items-start space-x-3">
              {/* Icon */}
              <div className="flex-shrink-0">
                <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
                  <Search className="w-4 h-4 text-gray-600" />
                </div>
              </div>

              {/* Content */}
              <div className="flex-1 min-w-0">
                <button
                  onClick={() => handleSearch(item)}
                  className="text-left w-full"
                >
                  <h3 className="text-sm font-medium text-gray-900 truncate">
                    {item.query}
                  </h3>
                  
                  <div className="mt-1 flex items-center space-x-4 text-xs text-gray-500">
                    <span className="flex items-center space-x-1">
                      <Calendar className="w-3 h-3" />
                      <span>{formatDate(item.timestamp)}</span>
                    </span>
                    <span>{formatTime(item.timestamp)}</span>
                    <span>{item.resultCount.toLocaleString()} results</span>
                  </div>

                  {/* Filters and Sort */}
                  {(item.filters || item.sort) && (
                    <div className="mt-2 flex flex-wrap gap-1">
                      {item.filters?.map((filter, index) => (
                        <span
                          key={index}
                          className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800"
                        >
                          <Filter className="w-3 h-3 mr-1" />
                          {filter}
                        </span>
                      ))}
                      {item.sort && (
                        <span className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-green-100 text-green-800">
                          <SortAsc className="w-3 h-3 mr-1" />
                          {item.sort}
                        </span>
                      )}
                    </div>
                  )}
                </button>
              </div>

              {/* Remove Button */}
              <button
                onClick={(e) => handleRemoveItem(item.id, e)}
                className="flex-shrink-0 p-1 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded transition-colors"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

export default MobileSearchHistory
