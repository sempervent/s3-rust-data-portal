// Mobile-optimized search bar component
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useRef, useEffect } from 'react'
import { Search, X, Filter, SortAsc } from 'lucide-react'
import { useSearchStore } from '@/stores/search'

interface MobileSearchBarProps {
  onToggleFilters?: () => void
  onToggleSort?: () => void
  showFilters?: boolean
  showSort?: boolean
}

export const MobileSearchBar: React.FC<MobileSearchBarProps> = ({
  onToggleFilters,
  onToggleSort,
  showFilters = false,
  showSort = false,
}) => {
  const { query, setQuery, search } = useSearchStore()
  const [isFocused, setIsFocused] = useState(false)
  const [showSuggestions, setShowSuggestions] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  // Handle search input
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value
    setQuery(value)
    
    // Show suggestions if there's text
    setShowSuggestions(value.length > 0)
  }

  // Handle search submission
  const handleSearch = async () => {
    if (query.trim()) {
      await search()
      setShowSuggestions(false)
      inputRef.current?.blur()
    }
  }

  // Handle Enter key
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch()
    } else if (e.key === 'Escape') {
      setShowSuggestions(false)
      inputRef.current?.blur()
    }
  }

  // Clear search
  const handleClear = () => {
    setQuery('')
    setShowSuggestions(false)
    inputRef.current?.focus()
  }

  // Focus input when component mounts
  useEffect(() => {
    if (isFocused) {
      inputRef.current?.focus()
    }
  }, [isFocused])

  return (
    <div className="w-full">
      {/* Search Input */}
      <div className="relative">
        <div className="flex items-center bg-white rounded-lg shadow-sm border border-gray-200 focus-within:border-blue-500 focus-within:ring-2 focus-within:ring-blue-200 transition-all duration-200">
          {/* Search Icon */}
          <div className="pl-3 pr-2">
            <Search className="w-5 h-5 text-gray-400" />
          </div>
          
          {/* Input Field */}
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={handleInputChange}
            onKeyDown={handleKeyPress}
            onFocus={() => setIsFocused(true)}
            onBlur={() => {
              setIsFocused(false)
              // Delay hiding suggestions to allow clicking
              setTimeout(() => setShowSuggestions(false), 200)
            }}
            placeholder="Search repositories, files, and data..."
            className="flex-1 py-3 px-2 text-gray-900 placeholder-gray-500 bg-transparent border-0 focus:outline-none focus:ring-0 text-base"
            autoComplete="off"
            autoCapitalize="off"
            autoCorrect="off"
            spellCheck="false"
          />
          
          {/* Clear Button */}
          {query && (
            <button
              onClick={handleClear}
              className="p-2 text-gray-400 hover:text-gray-600 transition-colors"
              aria-label="Clear search"
            >
              <X className="w-4 h-4" />
            </button>
          )}
          
          {/* Search Button */}
          <button
            onClick={handleSearch}
            disabled={!query.trim()}
            className="mr-2 p-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
            aria-label="Search"
          >
            <Search className="w-4 h-4" />
          </button>
        </div>
        
        {/* Suggestions Dropdown */}
        {showSuggestions && (
          <div className="absolute top-full left-0 right-0 mt-1 bg-white border border-gray-200 rounded-lg shadow-lg z-50 max-h-60 overflow-y-auto">
            {/* Recent Searches */}
            <div className="p-2">
              <div className="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">
                Recent Searches
              </div>
              <div className="space-y-1">
                {['machine learning', 'sales data', 'customer analytics'].map((suggestion, index) => (
                  <button
                    key={index}
                    onClick={() => {
                      setQuery(suggestion)
                      handleSearch()
                    }}
                    className="w-full text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-md transition-colors"
                  >
                    {suggestion}
                  </button>
                ))}
              </div>
            </div>
            
            {/* Quick Filters */}
            <div className="border-t border-gray-100 p-2">
              <div className="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">
                Quick Filters
              </div>
              <div className="grid grid-cols-2 gap-1">
                {[
                  { label: 'CSV Files', query: 'file_type:csv' },
                  { label: 'Images', query: 'file_type:image' },
                  { label: 'Documents', query: 'file_type:document' },
                  { label: 'Datasets', query: 'classification:public' },
                ].map((filter, index) => (
                  <button
                    key={index}
                    onClick={() => {
                      setQuery(filter.query)
                      handleSearch()
                    }}
                    className="text-left px-3 py-2 text-sm text-gray-700 hover:bg-gray-100 rounded-md transition-colors"
                  >
                    {filter.label}
                  </button>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
      
      {/* Action Buttons */}
      <div className="flex items-center justify-between mt-3">
        {/* Filter and Sort Buttons */}
        <div className="flex items-center space-x-2">
          {onToggleFilters && (
            <button
              onClick={onToggleFilters}
              className={`flex items-center px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                showFilters
                  ? 'bg-blue-100 text-blue-700 border border-blue-200'
                  : 'bg-gray-100 text-gray-700 border border-gray-200 hover:bg-gray-200'
              }`}
            >
              <Filter className="w-4 h-4 mr-1" />
              Filters
            </button>
          )}
          
          {onToggleSort && (
            <button
              onClick={onToggleSort}
              className={`flex items-center px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                showSort
                  ? 'bg-blue-100 text-blue-700 border border-blue-200'
                  : 'bg-gray-100 text-gray-700 border border-gray-200 hover:bg-gray-200'
              }`}
            >
              <SortAsc className="w-4 h-4 mr-1" />
              Sort
            </button>
          )}
        </div>
        
        {/* Search Stats */}
        <div className="text-sm text-gray-500">
          {/* This would show search results count */}
        </div>
      </div>
    </div>
  )
}

export default MobileSearchBar
