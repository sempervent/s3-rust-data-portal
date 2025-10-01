// Mobile-optimized search hooks
// Week 8: Mobile/responsive UX with PWA support

import { useState, useEffect, useCallback, useRef } from 'react'
import { useMobileSearchStore } from '@/stores/mobileSearch'
import { SearchResult, SearchQuery, SearchResponse } from '@/types/mobileSearch'
import { mobileSearchApi } from '@/services/mobileSearchApi'

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
      // Implement actual suggestions API call
      const response = await mobileSearchApi.getSuggestions({
        query: query.trim(),
        limit: 10
      })

      setSuggestions(response.suggestions)
      setTags(response.tags)
      setSearchSuggestions(response.suggestions)
      setSuggestedTags(response.tags)
    } catch (error) {
      console.error('Failed to fetch suggestions:', error)
      // Fallback to empty suggestions on error
      setSuggestions([])
      setTags([])
      setSearchSuggestions([])
      setSuggestedTags([])
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
      // Implement actual performance API call
      const response = await fetch('/api/v1/performance/metrics', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        throw new Error(`Performance API call failed: ${response.status}`)
      }
      
      const performanceData = await response.json()

      setPerformance(performanceData)
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
      // Implement actual alerts API call
      const response = await fetch('/api/v1/alerts', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        throw new Error(`Alerts API call failed: ${response.status}`)
      }
      
      const alertsData = await response.json()

      setAlerts(alertsData)
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
      // Implement actual export functionality
      const response = await fetch('/api/v1/export/search-results', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          format,
          query: searchQuery,
          filters: activeFilters,
          limit: 1000
        })
      })
      
      if (!response.ok) {
        throw new Error(`Export failed: ${response.status}`)
      }
      
      const blob = await response.blob()
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `search-results.${format}`
      document.body.appendChild(a)
      a.click()
      window.URL.revokeObjectURL(url)
      document.body.removeChild(a)
      
      console.log(`Export completed in ${format} format`)
    } catch (error) {
      console.error('Export failed:', error)
    } finally {
      setExporting(false)
    }
  }, [searchQuery, activeFilters])

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
      // Implement actual sharing functionality
      const response = await fetch('/api/v1/share/search-results', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          platform,
          query: searchQuery,
          filters: activeFilters,
          results: searchResults
        })
      })
      
      if (!response.ok) {
        throw new Error(`Sharing failed: ${response.status}`)
      }
      
      const shareData = await response.json()
      
      if (platform === 'link') {
        // Copy shareable link to clipboard
        await navigator.clipboard.writeText(shareData.shareUrl)
        console.log('Shareable link copied to clipboard')
      } else if (platform === 'email') {
        // Open email client with pre-filled content
        const subject = encodeURIComponent('Shared Search Results from BlackLake')
        const body = encodeURIComponent(`Check out these search results: ${shareData.shareUrl}`)
        window.open(`mailto:?subject=${subject}&body=${body}`)
      } else if (platform === 'social') {
        // Open social media sharing
        const text = encodeURIComponent(`Check out these search results from BlackLake: ${shareData.shareUrl}`)
        window.open(`https://twitter.com/intent/tweet?text=${text}`)
      }
      
      console.log(`Sharing completed via ${platform}`)
    } catch (error) {
      console.error('Sharing failed:', error)
    } finally {
      setSharing(false)
    }
  }, [searchQuery, activeFilters, searchResults])

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
      // Implement actual bookmarks API call
      const response = await fetch('/api/v1/bookmarks', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        throw new Error(`Bookmarks API call failed: ${response.status}`)
      }
      
      const bookmarksData = await response.json()
      setBookmarks(bookmarksData)
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
      // Implement actual bookmark creation
      const response = await fetch('/api/v1/bookmarks', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(bookmark)
      })
      
      if (!response.ok) {
        throw new Error(`Bookmark creation failed: ${response.status}`)
      }
      
      const newBookmark = await response.json()
      setBookmarks(prev => [newBookmark, ...prev])
    } catch (error) {
      console.error('Failed to add bookmark:', error)
    }
  }, [])

  const removeBookmark = useCallback(async (id: string) => {
    try {
      // Implement actual bookmark deletion
      const response = await fetch(`/api/v1/bookmarks/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        throw new Error(`Bookmark deletion failed: ${response.status}`)
      }
      
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
      // Implement actual notifications API call
      const response = await fetch('/api/v1/notifications', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        throw new Error(`Notifications API call failed: ${response.status}`)
      }
      
      const notificationsData = await response.json()
      setNotifications(notificationsData)
    } catch (error) {
      console.error('Failed to fetch notifications:', error)
    } finally {
      setLoading(false)
    }
  }, [])

  const markAsRead = useCallback(async (id: string) => {
    try {
      // Implement actual notification read status update
      const response = await fetch(`/api/v1/notifications/${id}/read`, {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        throw new Error(`Notification read update failed: ${response.status}`)
      }
      
      setNotifications(prev => prev.map(n => 
        n.id === id ? { ...n, read: true } : n
      ))
    } catch (error) {
      console.error('Failed to mark notification as read:', error)
    }
  }, [])

  const markAllAsRead = useCallback(async () => {
    try {
      // Implement actual bulk notification read status update
      const response = await fetch('/api/v1/notifications/read-all', {
        method: 'PUT',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        throw new Error(`Bulk notification read update failed: ${response.status}`)
      }
      
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
      // Implement actual context API call
      const response = await fetch('/api/v1/context', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('token')}`,
          'Content-Type': 'application/json'
        }
      })
      
      if (!response.ok) {
        throw new Error(`Context API call failed: ${response.status}`)
      }
      
      const contextData = await response.json()
      setContext(contextData)
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
