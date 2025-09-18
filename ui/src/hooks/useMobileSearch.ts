// Mobile-optimized search hooks
// Week 8: Mobile/responsive UX with PWA support

import { useState, useEffect, useCallback, useRef } from 'react'
import { useMobileSearchStore } from '@/stores/mobileSearch'
import { SearchResult, SearchQuery, SearchResponse } from '@/types/mobileSearch'

// Hook for search functionality
export const useSearch = () => {
  const {
    query,
    results,
    loading,
    totalCount,
    error,
    filters,
    sort,
    viewMode,
    semanticEnabled,
    search,
    loadMore,
    reset
  } = useMobileSearchStore()

  const [debouncedQuery, setDebouncedQuery] = useState(query)
  const [searchTimeout, setSearchTimeout] = useState<NodeJS.Timeout | null>(null)

  // Debounce search query
  useEffect(() => {
    if (searchTimeout) {
      clearTimeout(searchTimeout)
    }

    const timeout = setTimeout(() => {
      setDebouncedQuery(query)
    }, 300)

    setSearchTimeout(timeout)

    return () => {
      if (timeout) {
        clearTimeout(timeout)
      }
    }
  }, [query])

  // Auto-search when query changes
  useEffect(() => {
    if (debouncedQuery.trim()) {
      search(debouncedQuery, { filters, sort, semantic: semanticEnabled })
    }
  }, [debouncedQuery, filters, sort, semanticEnabled, search])

  const handleSearch = useCallback((searchQuery: string, options?: {
    filters?: Record<string, string[]>
    sort?: string
    semantic?: boolean
  }) => {
    search(searchQuery, options)
  }, [search])

  const handleLoadMore = useCallback(() => {
    loadMore()
  }, [loadMore])

  const handleReset = useCallback(() => {
    reset()
  }, [reset])

  return {
    query,
    results,
    loading,
    totalCount,
    error,
    filters,
    sort,
    viewMode,
    semanticEnabled,
    search: handleSearch,
    loadMore: handleLoadMore,
    reset: handleReset
  }
}

// Hook for search filters
export const useSearchFilters = () => {
  const {
    filters,
    setFilters,
    addFilter,
    removeFilter,
    clearFilters
  } = useMobileSearchStore()

  const handleAddFilter = useCallback((key: string, value: string) => {
    addFilter(key, value)
  }, [addFilter])

  const handleRemoveFilter = useCallback((key: string, value: string) => {
    removeFilter(key, value)
  }, [removeFilter])

  const handleClearFilters = useCallback(() => {
    clearFilters()
  }, [clearFilters])

  const handleSetFilters = useCallback((newFilters: Record<string, string[]>) => {
    setFilters(newFilters)
  }, [setFilters])

  return {
    filters,
    addFilter: handleAddFilter,
    removeFilter: handleRemoveFilter,
    clearFilters: handleClearFilters,
    setFilters: handleSetFilters
  }
}

// Hook for search sorting
export const useSearchSort = () => {
  const { sort, setSort } = useMobileSearchStore()

  const handleSetSort = useCallback((newSort: string) => {
    setSort(newSort)
  }, [setSort])

  return {
    sort,
    setSort: handleSetSort
  }
}

// Hook for search view mode
export const useSearchViewMode = () => {
  const { viewMode, setViewMode } = useMobileSearchStore()

  const handleSetViewMode = useCallback((mode: 'list' | 'grid') => {
    setViewMode(mode)
  }, [setViewMode])

  return {
    viewMode,
    setViewMode: handleSetViewMode
  }
}

// Hook for search history
export const useSearchHistory = () => {
  const {
    searchHistory,
    recentSearches,
    addToHistory,
    removeFromHistory,
    clearHistory
  } = useMobileSearchStore()

  const handleAddToHistory = useCallback((query: string) => {
    addToHistory(query)
  }, [addToHistory])

  const handleRemoveFromHistory = useCallback((query: string) => {
    removeFromHistory(query)
  }, [removeFromHistory])

  const handleClearHistory = useCallback(() => {
    clearHistory()
  }, [clearHistory])

  return {
    searchHistory,
    recentSearches,
    addToHistory: handleAddToHistory,
    removeFromHistory: handleRemoveFromHistory,
    clearHistory: handleClearHistory
  }
}

// Hook for search suggestions
export const useSearchSuggestions = () => {
  const {
    searchSuggestions,
    suggestedTags,
    setSearchSuggestions,
    setSuggestedTags
  } = useMobileSearchStore()

  const [suggestions, setSuggestions] = useState<string[]>([])
  const [tags, setTags] = useState<string[]>([])
  const [loading, setLoading] = useState(false)

  const fetchSuggestions = useCallback(async (query: string) => {
    if (!query.trim()) {
      setSuggestions([])
      setTags([])
      return
    }

    setLoading(true)
    try {
      // TODO: Implement actual suggestions API call
      // For now, return mock suggestions
      const mockSuggestions = [
        `${query} data`,
        `${query} analysis`,
        `${query} report`,
        `${query} metrics`
      ]

      const mockTags = [
        'analytics',
        'data',
        'report',
        'metrics'
      ]

      setSuggestions(mockSuggestions)
      setTags(mockTags)
      setSearchSuggestions(mockSuggestions)
      setSuggestedTags(mockTags)
    } catch (error) {
      console.error('Failed to fetch suggestions:', error)
    } finally {
      setLoading(false)
    }
  }, [setSearchSuggestions, setSuggestedTags])

  return {
    suggestions,
    tags,
    loading,
    fetchSuggestions
  }
}

// Hook for search analytics
export const useSearchAnalytics = () => {
  const {
    searchAnalytics,
    updateAnalytics,
    incrementSearchCount,
    addSearchTrend
  } = useMobileSearchStore()

  const handleUpdateAnalytics = useCallback((analytics: Partial<typeof searchAnalytics>) => {
    updateAnalytics(analytics)
  }, [updateAnalytics])

  const handleIncrementSearchCount = useCallback(() => {
    incrementSearchCount()
  }, [incrementSearchCount])

  const handleAddSearchTrend = useCallback((date: string, searches: number, users: number) => {
    addSearchTrend(date, searches, users)
  }, [addSearchTrend])

  return {
    analytics: searchAnalytics,
    updateAnalytics: handleUpdateAnalytics,
    incrementSearchCount: handleIncrementSearchCount,
    addSearchTrend: handleAddSearchTrend
  }
}

// Hook for search settings
export const useSearchSettings = () => {
  const {
    searchSettings,
    updateSettings,
    resetSettings
  } = useMobileSearchStore()

  const handleUpdateSettings = useCallback((settings: Partial<typeof searchSettings>) => {
    updateSettings(settings)
  }, [updateSettings])

  const handleResetSettings = useCallback(() => {
    resetSettings()
  }, [resetSettings])

  return {
    settings: searchSettings,
    updateSettings: handleUpdateSettings,
    resetSettings: handleResetSettings
  }
}

// Hook for search performance
export const useSearchPerformance = () => {
  const [performance, setPerformance] = useState({
    averageResponseTime: 0,
    p95ResponseTime: 0,
    p99ResponseTime: 0,
    throughput: 0,
    errorRate: 0,
    cacheHitRate: 0,
    indexSize: 0,
    queryComplexity: 0,
    memoryUsage: 0,
    cpuUsage: 0
  })

  const [loading, setLoading] = useState(false)

  const fetchPerformance = useCallback(async () => {
    setLoading(true)
    try {
      // TODO: Implement actual performance API call
      // For now, return mock performance data
      const mockPerformance = {
        averageResponseTime: 150,
        p95ResponseTime: 300,
        p99ResponseTime: 500,
        throughput: 1000,
        errorRate: 0.01,
        cacheHitRate: 0.85,
        indexSize: 1024 * 1024 * 1024, // 1GB
        queryComplexity: 0.7,
        memoryUsage: 0.6,
        cpuUsage: 0.4
      }

      setPerformance(mockPerformance)
    } catch (error) {
      console.error('Failed to fetch performance data:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchPerformance()
  }, [fetchPerformance])

  return {
    performance,
    loading,
    fetchPerformance
  }
}

// Hook for search alerts
export const useSearchAlerts = () => {
  const [alerts, setAlerts] = useState<Array<{
    id: string
    type: 'warning' | 'error' | 'info'
    message: string
    timestamp: string
    resolved: boolean
  }>>([])

  const [loading, setLoading] = useState(false)

  const fetchAlerts = useCallback(async () => {
    setLoading(true)
    try {
      // TODO: Implement actual alerts API call
      // For now, return mock alerts
      const mockAlerts = [
        {
          id: '1',
          type: 'warning' as const,
          message: 'High response time detected',
          timestamp: new Date().toISOString(),
          resolved: false
        },
        {
          id: '2',
          type: 'info' as const,
          message: 'Search index updated successfully',
          timestamp: new Date(Date.now() - 3600000).toISOString(),
          resolved: true
        }
      ]

      setAlerts(mockAlerts)
    } catch (error) {
      console.error('Failed to fetch alerts:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchAlerts()
  }, [fetchAlerts])

  return {
    alerts,
    loading,
    fetchAlerts
  }
}

// Hook for search export
export const useSearchExport = () => {
  const [exporting, setExporting] = useState(false)

  const exportResults = useCallback(async (format: 'json' | 'csv' | 'xlsx' | 'pdf') => {
    setExporting(true)
    try {
      // TODO: Implement actual export functionality
      // For now, just simulate export
      await new Promise(resolve => setTimeout(resolve, 2000))
      
      console.log(`Exporting results in ${format} format`)
    } catch (error) {
      console.error('Export failed:', error)
    } finally {
      setExporting(false)
    }
  }, [])

  return {
    exporting,
    exportResults
  }
}

// Hook for search sharing
export const useSearchShare = () => {
  const [sharing, setSharing] = useState(false)

  const shareResults = useCallback(async (platform: 'email' | 'social' | 'link') => {
    setSharing(true)
    try {
      // TODO: Implement actual sharing functionality
      // For now, just simulate sharing
      await new Promise(resolve => setTimeout(resolve, 1000))
      
      console.log(`Sharing results via ${platform}`)
    } catch (error) {
      console.error('Sharing failed:', error)
    } finally {
      setSharing(false)
    }
  }, [])

  return {
    sharing,
    shareResults
  }
}

// Hook for search bookmarks
export const useSearchBookmarks = () => {
  const [bookmarks, setBookmarks] = useState<Array<{
    id: string
    name: string
    query: string
    filters?: Record<string, string[]>
    sort?: string
    description?: string
    tags?: string[]
    createdAt: string
    updatedAt: string
  }>>([])

  const [loading, setLoading] = useState(false)

  const fetchBookmarks = useCallback(async () => {
    setLoading(true)
    try {
      // TODO: Implement actual bookmarks API call
      // For now, return mock bookmarks
      const mockBookmarks = [
        {
          id: '1',
          name: 'Customer Data Search',
          query: 'customer data',
          filters: { fileType: ['csv'] },
          sort: 'relevance',
          description: 'Search for customer data files',
          tags: ['customers', 'data'],
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString()
        }
      ]

      setBookmarks(mockBookmarks)
    } catch (error) {
      console.error('Failed to fetch bookmarks:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  const addBookmark = useCallback(async (bookmark: {
    name: string
    query: string
    filters?: Record<string, string[]>
    sort?: string
    description?: string
    tags?: string[]
  }) => {
    try {
      // TODO: Implement actual bookmark creation
      const newBookmark = {
        id: Math.random().toString(36).substr(2, 9),
        ...bookmark,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString()
      }

      setBookmarks(prev => [newBookmark, ...prev])
    } catch (error) {
      console.error('Failed to add bookmark:', error)
    }
  }, [])

  const removeBookmark = useCallback(async (id: string) => {
    try {
      // TODO: Implement actual bookmark deletion
      setBookmarks(prev => prev.filter(b => b.id !== id))
    } catch (error) {
      console.error('Failed to remove bookmark:', error)
    }
  }, [])

  useEffect(() => {
    fetchBookmarks()
  }, [fetchBookmarks])

  return {
    bookmarks,
    loading,
    fetchBookmarks,
    addBookmark,
    removeBookmark
  }
}

// Hook for search notifications
export const useSearchNotifications = () => {
  const [notifications, setNotifications] = useState<Array<{
    id: string
    type: 'new_results' | 'query_alert' | 'system_update'
    title: string
    message: string
    query?: string
    filters?: Record<string, string[]>
    timestamp: string
    read: boolean
    actionUrl?: string
  }>>([])

  const [loading, setLoading] = useState(false)

  const fetchNotifications = useCallback(async () => {
    setLoading(true)
    try {
      // TODO: Implement actual notifications API call
      // For now, return mock notifications
      const mockNotifications = [
        {
          id: '1',
          type: 'new_results' as const,
          title: 'New Results Available',
          message: 'New results found for your saved search "customer data"',
          query: 'customer data',
          timestamp: new Date().toISOString(),
          read: false
        }
      ]

      setNotifications(mockNotifications)
    } catch (error) {
      console.error('Failed to fetch notifications:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  const markAsRead = useCallback(async (id: string) => {
    try {
      // TODO: Implement actual notification read status update
      setNotifications(prev => prev.map(n => 
        n.id === id ? { ...n, read: true } : n
      ))
    } catch (error) {
      console.error('Failed to mark notification as read:', error)
    }
  }, [])

  const markAllAsRead = useCallback(async () => {
    try {
      // TODO: Implement actual bulk notification read status update
      setNotifications(prev => prev.map(n => ({ ...n, read: true })))
    } catch (error) {
      console.error('Failed to mark all notifications as read:', error)
    }
  }, [])

  useEffect(() => {
    fetchNotifications()
  }, [fetchNotifications])

  return {
    notifications,
    loading,
    fetchNotifications,
    markAsRead,
    markAllAsRead
  }
}

// Hook for search context
export const useSearchContext = () => {
  const [context, setContext] = useState({
    user: {
      id: '',
      name: '',
      email: '',
      role: '',
      permissions: [] as string[]
    },
    session: {
      id: '',
      startTime: '',
      lastActivity: '',
      ipAddress: '',
      userAgent: ''
    },
    environment: {
      version: '',
      build: '',
      environment: 'development' as 'development' | 'staging' | 'production',
      features: [] as string[]
    }
  })

  const [loading, setLoading] = useState(false)

  const fetchContext = useCallback(async () => {
    setLoading(true)
    try {
      // TODO: Implement actual context API call
      // For now, return mock context
      const mockContext = {
        user: {
          id: 'user-123',
          name: 'John Doe',
          email: 'john@example.com',
          role: 'user',
          permissions: ['read', 'search']
        },
        session: {
          id: 'session-456',
          startTime: new Date().toISOString(),
          lastActivity: new Date().toISOString(),
          ipAddress: '192.168.1.100',
          userAgent: 'Mozilla/5.0...'
        },
        environment: {
          version: '1.0.0',
          build: '123',
          environment: 'production' as const,
          features: ['semantic_search', 'filters', 'export']
        }
      }

      setContext(mockContext)
    } catch (error) {
      console.error('Failed to fetch context:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchContext()
  }, [fetchContext])

  return {
    context,
    loading,
    fetchContext
  }
}

// Hook for search error handling
export const useSearchError = () => {
  const [error, setError] = useState<string | null>(null)
  const [retryCount, setRetryCount] = useState(0)
  const [maxRetries] = useState(3)

  const handleError = useCallback((error: Error | string) => {
    const errorMessage = error instanceof Error ? error.message : error
    setError(errorMessage)
  }, [])

  const clearError = useCallback(() => {
    setError(null)
    setRetryCount(0)
  }, [])

  const retry = useCallback(() => {
    if (retryCount < maxRetries) {
      setRetryCount(prev => prev + 1)
      setError(null)
    }
  }, [retryCount, maxRetries])

  return {
    error,
    retryCount,
    maxRetries,
    handleError,
    clearError,
    retry
  }
}

// Hook for search debouncing
export const useSearchDebounce = (value: string, delay: number = 300) => {
  const [debouncedValue, setDebouncedValue] = useState(value)

  useEffect(() => {
    const handler = setTimeout(() => {
      setDebouncedValue(value)
    }, delay)

    return () => {
      clearTimeout(handler)
    }
  }, [value, delay])

  return debouncedValue
}

// Hook for search throttling
export const useSearchThrottle = (callback: Function, delay: number = 1000) => {
  const [isThrottled, setIsThrottled] = useState(false)
  const timeoutRef = useRef<NodeJS.Timeout | null>(null)

  const throttledCallback = useCallback((...args: any[]) => {
    if (!isThrottled) {
      callback(...args)
      setIsThrottled(true)
      
      timeoutRef.current = setTimeout(() => {
        setIsThrottled(false)
      }, delay)
    }
  }, [callback, delay, isThrottled])

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  return throttledCallback
}

// Hook for search pagination
export const useSearchPagination = (totalCount: number, pageSize: number = 25) => {
  const [currentPage, setCurrentPage] = useState(1)
  const [totalPages, setTotalPages] = useState(1)

  useEffect(() => {
    setTotalPages(Math.ceil(totalCount / pageSize))
  }, [totalCount, pageSize])

  const goToPage = useCallback((page: number) => {
    if (page >= 1 && page <= totalPages) {
      setCurrentPage(page)
    }
  }, [totalPages])

  const nextPage = useCallback(() => {
    if (currentPage < totalPages) {
      setCurrentPage(prev => prev + 1)
    }
  }, [currentPage, totalPages])

  const previousPage = useCallback(() => {
    if (currentPage > 1) {
      setCurrentPage(prev => prev - 1)
    }
  }, [currentPage])

  const resetPagination = useCallback(() => {
    setCurrentPage(1)
  }, [])

  return {
    currentPage,
    totalPages,
    goToPage,
    nextPage,
    previousPage,
    resetPagination
  }
}

// Hook for search infinite scroll
export const useSearchInfiniteScroll = (
  hasMore: boolean,
  loading: boolean,
  onLoadMore: () => void
) => {
  const [isIntersecting, setIsIntersecting] = useState(false)
  const sentinelRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!sentinelRef.current) return

    const observer = new IntersectionObserver(
      (entries) => {
        const [entry] = entries
        setIsIntersecting(entry.isIntersecting)
      },
      {
        threshold: 0.1,
        rootMargin: '100px'
      }
    )

    observer.observe(sentinelRef.current)

    return () => {
      observer.disconnect()
    }
  }, [])

  useEffect(() => {
    if (isIntersecting && hasMore && !loading) {
      onLoadMore()
    }
  }, [isIntersecting, hasMore, loading, onLoadMore])

  return { sentinelRef }
}

export default {
  useSearch,
  useSearchFilters,
  useSearchSort,
  useSearchViewMode,
  useSearchHistory,
  useSearchSuggestions,
  useSearchAnalytics,
  useSearchSettings,
  useSearchPerformance,
  useSearchAlerts,
  useSearchExport,
  useSearchShare,
  useSearchBookmarks,
  useSearchNotifications,
  useSearchContext,
  useSearchError,
  useSearchDebounce,
  useSearchThrottle,
  useSearchPagination,
  useSearchInfiniteScroll
}
