// Mobile-optimized search suggestions component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useCallback, useEffect } from 'react'
import { 
  Search, 
  Clock, 
  TrendingUp, 
  Tag, 
  File, 
  Folder,
  X,
  ArrowUp,
  ArrowDown
} from 'lucide-react'

interface SearchSuggestion {
  id: string
  text: string
  type: 'recent' | 'trending' | 'tag' | 'file' | 'folder'
  count?: number
  lastUsed?: string
}

interface MobileSearchSuggestionsProps {
  query: string
  suggestions: SearchSuggestion[]
  onSuggestionSelect: (suggestion: string) => void
  onClearRecent: () => void
  className?: string
}

export const MobileSearchSuggestions: React.FC<MobileSearchSuggestionsProps> = ({
  query,
  suggestions,
  onSuggestionSelect,
  onClearRecent,
  className = ''
}) => {
  const [selectedIndex, setSelectedIndex] = useState(-1)
  const [isVisible, setIsVisible] = useState(false)

  // Show suggestions when there's a query
  useEffect(() => {
    setIsVisible(query.length > 0 && suggestions.length > 0)
  }, [query, suggestions])

  // Reset selected index when suggestions change
  useEffect(() => {
    setSelectedIndex(-1)
  }, [suggestions])

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    if (!isVisible || suggestions.length === 0) return

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setSelectedIndex(prev => 
          prev < suggestions.length - 1 ? prev + 1 : 0
        )
        break
      case 'ArrowUp':
        e.preventDefault()
        setSelectedIndex(prev => 
          prev > 0 ? prev - 1 : suggestions.length - 1
        )
        break
      case 'Enter':
        e.preventDefault()
        if (selectedIndex >= 0 && selectedIndex < suggestions.length) {
          onSuggestionSelect(suggestions[selectedIndex].text)
        }
        break
      case 'Escape':
        setIsVisible(false)
        break
    }
  }, [isVisible, suggestions, selectedIndex, onSuggestionSelect])

  // Handle suggestion click
  const handleSuggestionClick = useCallback((suggestion: SearchSuggestion) => {
    onSuggestionSelect(suggestion.text)
  }, [onSuggestionSelect])

  // Get suggestion icon
  const getSuggestionIcon = (type: string) => {
    switch (type) {
      case 'recent':
        return <Clock className="w-4 h-4 text-gray-400" />
      case 'trending':
        return <TrendingUp className="w-4 h-4 text-green-500" />
      case 'tag':
        return <Tag className="w-4 h-4 text-blue-500" />
      case 'file':
        return <File className="w-4 h-4 text-gray-500" />
      case 'folder':
        return <Folder className="w-4 h-4 text-blue-500" />
      default:
        return <Search className="w-4 h-4 text-gray-400" />
    }
  }

  // Get suggestion type label
  const getSuggestionTypeLabel = (type: string) => {
    switch (type) {
      case 'recent':
        return 'Recent'
      case 'trending':
        return 'Trending'
      case 'tag':
        return 'Tag'
      case 'file':
        return 'File'
      case 'folder':
        return 'Folder'
      default:
        return 'Suggestion'
    }
  }

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

  // Group suggestions by type
  const groupedSuggestions = suggestions.reduce((groups, suggestion) => {
    const type = suggestion.type
    if (!groups[type]) {
      groups[type] = []
    }
    groups[type].push(suggestion)
    return groups
  }, {} as Record<string, SearchSuggestion[]>)

  if (!isVisible) return null

  return (
    <div
      className={`absolute top-full left-0 right-0 mt-1 bg-white border border-gray-200 rounded-lg shadow-lg z-50 max-h-80 overflow-y-auto ${className}`}
      onKeyDown={handleKeyDown}
    >
      {/* Header */}
      <div className="flex items-center justify-between p-3 border-b border-gray-100">
        <span className="text-sm font-medium text-gray-700">
          Suggestions
        </span>
        <button
          onClick={onClearRecent}
          className="text-xs text-gray-500 hover:text-gray-700 transition-colors"
        >
          Clear recent
        </button>
      </div>

      {/* Suggestions */}
      <div className="py-2">
        {Object.entries(groupedSuggestions).map(([type, typeSuggestions]) => (
          <div key={type} className="mb-4 last:mb-0">
            {/* Type Header */}
            <div className="px-3 py-2">
              <div className="flex items-center space-x-2">
                {getSuggestionIcon(type)}
                <span className="text-xs font-medium text-gray-500 uppercase tracking-wide">
                  {getSuggestionTypeLabel(type)}
                </span>
              </div>
            </div>

            {/* Type Suggestions */}
            <div className="space-y-1">
              {typeSuggestions.map((suggestion, index) => {
                const globalIndex = suggestions.indexOf(suggestion)
                const isSelected = globalIndex === selectedIndex
                
                return (
                  <button
                    key={suggestion.id}
                    onClick={() => handleSuggestionClick(suggestion)}
                    className={`w-full flex items-center justify-between px-3 py-2 text-left hover:bg-gray-50 transition-colors ${
                      isSelected ? 'bg-blue-50 text-blue-700' : 'text-gray-700'
                    }`}
                  >
                    <div className="flex items-center space-x-3 flex-1 min-w-0">
                      <div className="flex-shrink-0">
                        {getSuggestionIcon(suggestion.type)}
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="text-sm font-medium truncate">
                          {suggestion.text}
                        </p>
                        {suggestion.lastUsed && (
                          <p className="text-xs text-gray-500">
                            {formatDate(suggestion.lastUsed)}
                          </p>
                        )}
                      </div>
                    </div>
                    
                    <div className="flex items-center space-x-2 flex-shrink-0">
                      {suggestion.count && (
                        <span className="text-xs text-gray-500">
                          {suggestion.count.toLocaleString()}
                        </span>
                      )}
                      {isSelected && (
                        <div className="flex items-center space-x-1">
                          <ArrowUp className="w-3 h-3" />
                          <ArrowDown className="w-3 h-3" />
                        </div>
                      )}
                    </div>
                  </button>
                )
              })}
            </div>
          </div>
        ))}
      </div>

      {/* Footer */}
      <div className="px-3 py-2 border-t border-gray-100">
        <div className="flex items-center justify-between text-xs text-gray-500">
          <span>Use ↑↓ to navigate</span>
          <span>Enter to select</span>
        </div>
      </div>
    </div>
  )
}

export default MobileSearchSuggestions
