import React, { useState, useEffect, useRef } from 'react'
import { useApi } from '@/hooks/useApi'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Card } from '@/components/ui/card'
import { Search, X, Filter } from 'lucide-react'

interface Suggestion {
  term: string
  count?: number
}

interface SearchBarProps {
  onSearch: (query: string, filters: SearchFilters) => void
  onFiltersChange: (filters: SearchFilters) => void
  initialQuery?: string
  initialFilters?: SearchFilters
  showAdvanced?: boolean
}

interface SearchFilters {
  query: string
  file_type: string[]
  org_lab: string[]
  tags: string[]
  size_min: string
  size_max: string
  date_from: string
  date_to: string
}

const EnhancedSearchBar: React.FC<SearchBarProps> = ({
  onSearch,
  onFiltersChange,
  initialQuery = '',
  initialFilters = {
    query: '',
    file_type: [],
    org_lab: [],
    tags: [],
    size_min: '',
    size_max: '',
    date_from: '',
    date_to: ''
  },
  showAdvanced = false
}) => {
  const [query, setQuery] = useState(initialQuery)
  const [filters, setFilters] = useState<SearchFilters>(initialFilters)
  const [suggestions, setSuggestions] = useState<Suggestion[]>([])
  const [showSuggestions, setShowSuggestions] = useState(false)
  const [showAdvancedFilters, setShowAdvancedFilters] = useState(showAdvanced)
  const [isSearching, setIsSearching] = useState(false)
  
  const inputRef = useRef<HTMLInputElement>(null)
  const suggestionsRef = useRef<HTMLDivElement>(null)

  // Debounced search for suggestions
  const { data: suggestionData } = useApi<{ suggestions: string[] }>(
    `/v1/search/suggest?q=${encodeURIComponent(query)}&count=5`,
    { 
      enabled: query.length > 2,
      debounceMs: 300
    }
  )

  // Update suggestions when data changes
  useEffect(() => {
    if (suggestionData?.suggestions) {
      setSuggestions(suggestionData.suggestions.map(term => ({ term })))
    }
  }, [suggestionData])

  // Handle input change
  const handleInputChange = (value: string) => {
    setQuery(value)
    setShowSuggestions(value.length > 2)
  }

  // Handle suggestion selection
  const handleSuggestionSelect = (suggestion: string) => {
    setQuery(suggestion)
    setShowSuggestions(false)
    handleSearch(suggestion)
  }

  // Handle search
  const handleSearch = (searchQuery?: string) => {
    const finalQuery = searchQuery || query
    if (finalQuery.trim()) {
      setIsSearching(true)
      onSearch(finalQuery, filters)
      setShowSuggestions(false)
    }
  }

  // Handle key press
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch()
    } else if (e.key === 'Escape') {
      setShowSuggestions(false)
    }
  }

  // Handle filter change
  const handleFilterChange = (key: keyof SearchFilters, value: any) => {
    const newFilters = { ...filters, [key]: value }
    setFilters(newFilters)
    onFiltersChange(newFilters)
  }

  // Clear filters
  const clearFilters = () => {
    const clearedFilters = {
      query: '',
      file_type: [],
      org_lab: [],
      tags: [],
      size_min: '',
      size_max: '',
      date_from: '',
      date_to: ''
    }
    setFilters(clearedFilters)
    setQuery('')
    onFiltersChange(clearedFilters)
  }

  // Close suggestions when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (suggestionsRef.current && !suggestionsRef.current.contains(event.target as Node) &&
          inputRef.current && !inputRef.current.contains(event.target as Node)) {
        setShowSuggestions(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  return (
    <div className="relative w-full">
      {/* Main Search Bar */}
      <div className="flex gap-2">
        <div className="relative flex-1">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400 h-4 w-4" />
            <Input
              ref={inputRef}
              type="text"
              placeholder="Search repositories, files, and metadata..."
              value={query}
              onChange={(e) => handleInputChange(e.target.value)}
              onKeyPress={handleKeyPress}
              onFocus={() => query.length > 2 && setShowSuggestions(true)}
              className="pl-10 pr-4 py-2 w-full"
            />
            {query && (
              <button
                onClick={() => {
                  setQuery('')
                  setShowSuggestions(false)
                }}
                className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
              >
                <X className="h-4 w-4" />
              </button>
            )}
          </div>

          {/* Suggestions Dropdown */}
          {showSuggestions && suggestions.length > 0 && (
            <Card ref={suggestionsRef} className="absolute top-full left-0 right-0 mt-1 z-50 max-h-60 overflow-y-auto">
              {suggestions.map((suggestion, index) => (
                <button
                  key={index}
                  onClick={() => handleSuggestionSelect(suggestion.term)}
                  className="w-full px-4 py-2 text-left hover:bg-gray-50 border-b last:border-b-0"
                >
                  <div className="flex items-center gap-2">
                    <Search className="h-4 w-4 text-gray-400" />
                    <span>{suggestion.term}</span>
                    {suggestion.count && (
                      <span className="text-sm text-gray-500 ml-auto">
                        {suggestion.count} results
                      </span>
                    )}
                  </div>
                </button>
              ))}
            </Card>
          )}
        </div>

        <Button
          onClick={() => handleSearch()}
          disabled={!query.trim() || isSearching}
          className="px-6"
        >
          {isSearching ? 'Searching...' : 'Search'}
        </Button>

        <Button
          variant="outline"
          onClick={() => setShowAdvancedFilters(!showAdvancedFilters)}
          className="px-4"
        >
          <Filter className="h-4 w-4 mr-2" />
          Filters
        </Button>
      </div>

      {/* Advanced Filters */}
      {showAdvancedFilters && (
        <Card className="mt-4 p-4">
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
            {/* File Type Filter */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                File Type
              </label>
              <Input
                type="text"
                placeholder="e.g., csv, parquet, json"
                value={filters.file_type.join(', ')}
                onChange={(e) => handleFilterChange('file_type', e.target.value.split(',').map(s => s.trim()).filter(Boolean))}
              />
            </div>

            {/* Organization/Lab Filter */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Organization/Lab
              </label>
              <Input
                type="text"
                placeholder="e.g., ornl, lanl"
                value={filters.org_lab.join(', ')}
                onChange={(e) => handleFilterChange('org_lab', e.target.value.split(',').map(s => s.trim()).filter(Boolean))}
              />
            </div>

            {/* Tags Filter */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Tags
              </label>
              <Input
                type="text"
                placeholder="e.g., ml, experiment"
                value={filters.tags.join(', ')}
                onChange={(e) => handleFilterChange('tags', e.target.value.split(',').map(s => s.trim()).filter(Boolean))}
              />
            </div>

            {/* Size Range */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                File Size (MB)
              </label>
              <div className="flex gap-2">
                <Input
                  type="number"
                  placeholder="Min"
                  value={filters.size_min}
                  onChange={(e) => handleFilterChange('size_min', e.target.value)}
                  className="flex-1"
                />
                <Input
                  type="number"
                  placeholder="Max"
                  value={filters.size_max}
                  onChange={(e) => handleFilterChange('size_max', e.target.value)}
                  className="flex-1"
                />
              </div>
            </div>

            {/* Date Range */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Date Range
              </label>
              <div className="flex gap-2">
                <Input
                  type="date"
                  value={filters.date_from}
                  onChange={(e) => handleFilterChange('date_from', e.target.value)}
                  className="flex-1"
                />
                <Input
                  type="date"
                  value={filters.date_to}
                  onChange={(e) => handleFilterChange('date_to', e.target.value)}
                  className="flex-1"
                />
              </div>
            </div>
          </div>

          {/* Filter Actions */}
          <div className="flex justify-between items-center mt-4 pt-4 border-t">
            <Button
              variant="outline"
              onClick={clearFilters}
              className="text-sm"
            >
              Clear All Filters
            </Button>
            <div className="text-sm text-gray-500">
              {Object.values(filters).filter(v => 
                Array.isArray(v) ? v.length > 0 : v !== ''
              ).length} filters active
            </div>
          </div>
        </Card>
      )}
    </div>
  )
}

export default EnhancedSearchBar
