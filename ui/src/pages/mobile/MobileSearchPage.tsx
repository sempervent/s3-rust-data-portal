// Mobile-optimized search page
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect, useCallback } from 'react'
import { useSearchParams } from 'react-router-dom'
import MobileSearchBar from '@/components/mobile/MobileSearchBar'
import MobileSearchResults from '@/components/mobile/MobileSearchResults'
import { useSearchStore } from '@/stores/search'
import { SearchResult } from '@/types/search'

const MobileSearchPage: React.FC = () => {
  const [searchParams, setSearchParams] = useSearchParams()
  const { query, setQuery, search, results, loading, totalCount } = useSearchStore()
  const [showFilters, setShowFilters] = useState(false)
  const [showSort, setShowSort] = useState(false)
  const [page, setPage] = useState(1)

  // Initialize search from URL params
  useEffect(() => {
    const urlQuery = searchParams.get('q')
    if (urlQuery && urlQuery !== query) {
      setQuery(urlQuery)
    }
  }, [searchParams, query, setQuery])

  // Update URL when query changes
  useEffect(() => {
    if (query) {
      setSearchParams({ q: query })
    } else {
      setSearchParams({})
    }
  }, [query, setSearchParams])

  // Handle search
  const handleSearch = useCallback(async () => {
    if (query.trim()) {
      setPage(1)
      await search()
    }
  }, [query, search])

  // Handle load more
  const handleLoadMore = useCallback(async () => {
    if (!loading && results.length < totalCount) {
      setPage(prev => prev + 1)
      // TODO: Implement pagination in search store
    }
  }, [loading, results.length, totalCount])

  // Handle result click
  const handleResultClick = useCallback((result: SearchResult) => {
    // Navigate to result details or open file
    if (result.type === 'directory') {
      // Navigate to directory
      window.location.href = `/repositories${result.path}`
    } else {
      // Open file or show details
      window.location.href = `/repositories${result.path}`
    }
  }, [])

  // Handle filter toggle
  const handleToggleFilters = useCallback(() => {
    setShowFilters(!showFilters)
    setShowSort(false)
  }, [showFilters])

  // Handle sort toggle
  const handleToggleSort = useCallback(() => {
    setShowSort(!showSort)
    setShowFilters(false)
  }, [showSort])

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Search Header */}
      <div className="bg-white border-b border-gray-200 px-4 py-3">
        <MobileSearchBar
          onToggleFilters={handleToggleFilters}
          onToggleSort={handleToggleSort}
          showFilters={showFilters}
          showSort={showSort}
        />
      </div>

      {/* Filters Panel */}
      {showFilters && (
        <div className="bg-white border-b border-gray-200 p-4">
          <div className="space-y-4">
            {/* File Type Filter */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                File Type
              </label>
              <div className="grid grid-cols-2 gap-2">
                {['CSV', 'JSON', 'PDF', 'Images'].map((type) => (
                  <button
                    key={type}
                    className="px-3 py-2 text-sm border border-gray-200 rounded-md hover:bg-gray-50 transition-colors"
                  >
                    {type}
                  </button>
                ))}
              </div>
            </div>

            {/* Repository Filter */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Repository
              </label>
              <select className="w-full px-3 py-2 border border-gray-200 rounded-md text-sm">
                <option value="">All Repositories</option>
                <option value="repo1">Repository 1</option>
                <option value="repo2">Repository 2</option>
              </select>
            </div>

            {/* Date Range Filter */}
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Date Range
              </label>
              <div className="grid grid-cols-2 gap-2">
                {['Last 7 days', 'Last 30 days', 'Last 3 months', 'All time'].map((range) => (
                  <button
                    key={range}
                    className="px-3 py-2 text-sm border border-gray-200 rounded-md hover:bg-gray-50 transition-colors"
                  >
                    {range}
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Sort Panel */}
      {showSort && (
        <div className="bg-white border-b border-gray-200 p-4">
          <div className="space-y-2">
            {[
              { label: 'Relevance', value: 'relevance' },
              { label: 'Name A-Z', value: 'name_asc' },
              { label: 'Name Z-A', value: 'name_desc' },
              { label: 'Date Modified', value: 'date_desc' },
              { label: 'Size', value: 'size_desc' },
            ].map((option) => (
              <button
                key={option.value}
                className="w-full text-left px-3 py-2 text-sm hover:bg-gray-50 rounded-md transition-colors"
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Search Results */}
      <MobileSearchResults
        results={results}
        totalCount={totalCount}
        loading={loading}
        onResultClick={handleResultClick}
        onLoadMore={handleLoadMore}
      />

      {/* Empty State */}
      {!loading && results.length === 0 && query && (
        <div className="p-8 text-center">
          <div className="text-gray-500">
            <p className="text-sm">No results found for "{query}"</p>
            <p className="text-xs mt-2">Try adjusting your search terms or filters</p>
          </div>
        </div>
      )}

      {/* Initial State */}
      {!query && (
        <div className="p-8 text-center">
          <div className="text-gray-500">
            <p className="text-sm">Search for repositories, files, and data</p>
            <p className="text-xs mt-2">Use the search bar above to get started</p>
          </div>
        </div>
      )}
    </div>
  )
}

export default MobileSearchPage
