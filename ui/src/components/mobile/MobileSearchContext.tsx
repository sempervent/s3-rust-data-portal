// Mobile-optimized search context component
// Week 8: Mobile/responsive UX with PWA support

import React, { createContext, useContext, useState, useCallback, useEffect } from 'react'
import { SearchResult } from '@/types/search'

interface SearchContextState {
  query: string
  results: SearchResult[]
  loading: boolean
  totalCount: number
  filters: Record<string, string[]>
  sort: string
  viewMode: 'list' | 'grid'
  semanticEnabled: boolean
  searchHistory: string[]
  recentSearches: string[]
  popularSearches: string[]
  searchSuggestions: string[]
  searchAnalytics: {
    totalSearches: number
    uniqueUsers: number
    averageResponseTime: number
    topQueries: Array<{
      query: string
      count: number
      trend: 'up' | 'down' | 'stable'
    }>
  }
  searchSettings: {
    enableSemanticSearch: boolean
    enableAutoComplete: boolean
    enableSearchHistory: boolean
    enableSearchSuggestions: boolean
    enableSearchAnalytics: boolean
    defaultSearchMode: 'keyword' | 'semantic' | 'hybrid'
    resultsPerPage: number
  }
}

interface SearchContextActions {
  setQuery: (query: string) => void
  setResults: (results: SearchResult[]) => void
  setLoading: (loading: boolean) => void
  setTotalCount: (count: number) => void
  setFilters: (filters: Record<string, string[]>) => void
  setSort: (sort: string) => void
  setViewMode: (mode: 'list' | 'grid') => void
  setSemanticEnabled: (enabled: boolean) => void
  addToHistory: (query: string) => void
  clearHistory: () => void
  removeFromHistory: (query: string) => void
  updateAnalytics: (analytics: Partial<SearchContextState['searchAnalytics']>) => void
  updateSettings: (settings: Partial<SearchContextState['searchSettings']>) => void
  search: (query: string, options?: {
    filters?: Record<string, string[]>
    sort?: string
    semantic?: boolean
  }) => Promise<void>
  loadMore: () => Promise<void>
  reset: () => void
}

type SearchContextType = SearchContextState & SearchContextActions

const SearchContext = createContext<SearchContextType | undefined>(undefined)

export const useSearchContext = () => {
  const context = useContext(SearchContext)
  if (!context) {
    throw new Error('useSearchContext must be used within a SearchProvider')
  }
  return context
}

interface SearchProviderProps {
  children: React.ReactNode
}

export const SearchProvider: React.FC<SearchProviderProps> = ({ children }) => {
  // State
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchResult[]>([])
  const [loading, setLoading] = useState(false)
  const [totalCount, setTotalCount] = useState(0)
  const [filters, setFilters] = useState<Record<string, string[]>>({})
  const [sort, setSort] = useState('relevance')
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list')
  const [semanticEnabled, setSemanticEnabled] = useState(true)
  const [searchHistory, setSearchHistory] = useState<string[]>([])
  const [recentSearches, setRecentSearches] = useState<string[]>([])
  const [popularSearches, setPopularSearches] = useState<string[]>([])
  const [searchSuggestions, setSearchSuggestions] = useState<string[]>([])
  const [searchAnalytics, setSearchAnalytics] = useState<SearchContextState['searchAnalytics']>({
    totalSearches: 0,
    uniqueUsers: 0,
    averageResponseTime: 0,
    topQueries: []
  })
  const [searchSettings, setSearchSettings] = useState<SearchContextState['searchSettings']>({
    enableSemanticSearch: true,
    enableAutoComplete: true,
    enableSearchHistory: true,
    enableSearchSuggestions: true,
    enableSearchAnalytics: true,
    defaultSearchMode: 'hybrid',
    resultsPerPage: 25
  })

  // Load initial data
  useEffect(() => {
    const loadInitialData = async () => {
      try {
        // Load search history from localStorage
        const savedHistory = localStorage.getItem('searchHistory')
        if (savedHistory) {
          setSearchHistory(JSON.parse(savedHistory))
        }

        // Load recent searches
        const savedRecent = localStorage.getItem('recentSearches')
        if (savedRecent) {
          setRecentSearches(JSON.parse(savedRecent))
        }

        // Load popular searches
        const savedPopular = localStorage.getItem('popularSearches')
        if (savedPopular) {
          setPopularSearches(JSON.parse(savedPopular))
        }

        // Load search settings
        const savedSettings = localStorage.getItem('searchSettings')
        if (savedSettings) {
          setSearchSettings(JSON.parse(savedSettings))
        }

        // Load search analytics
        const savedAnalytics = localStorage.getItem('searchAnalytics')
        if (savedAnalytics) {
          setSearchAnalytics(JSON.parse(savedAnalytics))
        }
      } catch (error) {
        console.error('Failed to load initial search data:', error)
      }
    }

    loadInitialData()
  }, [])

  // Save data to localStorage
  useEffect(() => {
    localStorage.setItem('searchHistory', JSON.stringify(searchHistory))
  }, [searchHistory])

  useEffect(() => {
    localStorage.setItem('recentSearches', JSON.stringify(recentSearches))
  }, [recentSearches])

  useEffect(() => {
    localStorage.setItem('popularSearches', JSON.stringify(popularSearches))
  }, [popularSearches])

  useEffect(() => {
    localStorage.setItem('searchSettings', JSON.stringify(searchSettings))
  }, [searchSettings])

  useEffect(() => {
    localStorage.setItem('searchAnalytics', JSON.stringify(searchAnalytics))
  }, [searchAnalytics])

  // Actions
  const addToHistory = useCallback((newQuery: string) => {
    if (!newQuery.trim()) return

    setSearchHistory(prev => {
      const filtered = prev.filter(q => q !== newQuery)
      return [newQuery, ...filtered].slice(0, 50) // Keep last 50 searches
    })

    setRecentSearches(prev => {
      const filtered = prev.filter(q => q !== newQuery)
      return [newQuery, ...filtered].slice(0, 10) // Keep last 10 recent searches
    })
  }, [])

  const clearHistory = useCallback(() => {
    setSearchHistory([])
    setRecentSearches([])
  }, [])

  const removeFromHistory = useCallback((queryToRemove: string) => {
    setSearchHistory(prev => prev.filter(q => q !== queryToRemove))
    setRecentSearches(prev => prev.filter(q => q !== queryToRemove))
  }, [])

  const updateAnalytics = useCallback((newAnalytics: Partial<SearchContextState['searchAnalytics']>) => {
    setSearchAnalytics(prev => ({ ...prev, ...newAnalytics }))
  }, [])

  const updateSettings = useCallback((newSettings: Partial<SearchContextState['searchSettings']>) => {
    setSearchSettings(prev => ({ ...prev, ...newSettings }))
  }, [])

  const search = useCallback(async (searchQuery: string, options?: {
    filters?: Record<string, string[]>
    sort?: string
    semantic?: boolean
  }) => {
    if (!searchQuery.trim()) return

    setLoading(true)
    setQuery(searchQuery)
    
    if (options?.filters) {
      setFilters(options.filters)
    }
    
    if (options?.sort) {
      setSort(options.sort)
    }
    
    if (options?.semantic !== undefined) {
      setSemanticEnabled(options.semantic)
    }

    try {
      // Add to history
      addToHistory(searchQuery)

      // Update analytics
      updateAnalytics({
        totalSearches: searchAnalytics.totalSearches + 1
      })

      // TODO: Implement actual search API call
      // For now, return mock results
      const mockResults: SearchResult[] = [
        {
          id: '1',
          name: 'customer_data.csv',
          path: '/datasets/customer_data.csv',
          description: 'Customer information and analytics data',
          type: 'file',
          size: 1024000,
          lastModified: new Date().toISOString(),
          author: 'John Doe',
          tags: ['customers', 'analytics', 'data'],
          repository: 'main'
        },
        {
          id: '2',
          name: 'sales_report.pdf',
          path: '/reports/sales_report.pdf',
          description: 'Monthly sales performance report',
          type: 'file',
          size: 512000,
          lastModified: new Date().toISOString(),
          author: 'Jane Smith',
          tags: ['sales', 'report', 'performance'],
          repository: 'main'
        }
      ]

      setResults(mockResults)
      setTotalCount(mockResults.length)
    } catch (error) {
      console.error('Search failed:', error)
      setResults([])
      setTotalCount(0)
    } finally {
      setLoading(false)
    }
  }, [addToHistory, updateAnalytics, searchAnalytics.totalSearches])

  const loadMore = useCallback(async () => {
    if (loading || results.length >= totalCount) return

    setLoading(true)
    try {
      // TODO: Implement pagination
      // For now, just simulate loading more results
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      // Mock additional results
      const additionalResults: SearchResult[] = [
        {
          id: `${results.length + 1}`,
          name: `additional_file_${results.length + 1}.csv`,
          path: `/datasets/additional_file_${results.length + 1}.csv`,
          description: 'Additional search result',
          type: 'file',
          size: 256000,
          lastModified: new Date().toISOString(),
          author: 'System',
          tags: ['additional', 'data'],
          repository: 'main'
        }
      ]

      setResults(prev => [...prev, ...additionalResults])
    } catch (error) {
      console.error('Load more failed:', error)
    } finally {
      setLoading(false)
    }
  }, [loading, results.length, totalCount])

  const reset = useCallback(() => {
    setQuery('')
    setResults([])
    setLoading(false)
    setTotalCount(0)
    setFilters({})
    setSort('relevance')
    setViewMode('list')
    setSemanticEnabled(true)
  }, [])

  // Context value
  const contextValue: SearchContextType = {
    // State
    query,
    results,
    loading,
    totalCount,
    filters,
    sort,
    viewMode,
    semanticEnabled,
    searchHistory,
    recentSearches,
    popularSearches,
    searchSuggestions,
    searchAnalytics,
    searchSettings,
    
    // Actions
    setQuery,
    setResults,
    setLoading,
    setTotalCount,
    setFilters,
    setSort,
    setViewMode,
    setSemanticEnabled,
    addToHistory,
    clearHistory,
    removeFromHistory,
    updateAnalytics,
    updateSettings,
    search,
    loadMore,
    reset
  }

  return (
    <SearchContext.Provider value={contextValue}>
      {children}
    </SearchContext.Provider>
  )
}

export default SearchProvider
