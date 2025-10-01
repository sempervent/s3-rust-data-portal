// Mobile-optimized search store
// Week 8: Mobile/responsive UX with PWA support

import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import { SearchResult } from '@/types/search'
import { mobileSearchApi } from '@/services/mobileSearchApi'

interface SearchState {
  // Search state
  query: string
  results: SearchResult[]
  loading: boolean
  totalCount: number
  error: string | null
  
  // Filters and sorting
  filters: Record<string, string[]>
  sort: string
  viewMode: 'list' | 'grid'
  
  // Search features
  semanticEnabled: boolean
  autoCompleteEnabled: boolean
  searchHistoryEnabled: boolean
  searchSuggestionsEnabled: boolean
  
  // Search history
  searchHistory: string[]
  recentSearches: string[]
  popularSearches: string[]
  
  // Search suggestions
  searchSuggestions: string[]
  suggestedTags: string[]
  
  // Search analytics
  searchAnalytics: {
    totalSearches: number
    uniqueUsers: number
    averageResponseTime: number
    topQueries: Array<{
      query: string
      count: number
      trend: 'up' | 'down' | 'stable'
    }>
    searchTrends: Array<{
      date: string
      searches: number
      users: number
    }>
  }
  
  // Search settings
  searchSettings: {
    enableSemanticSearch: boolean
    enableAutoComplete: boolean
    enableSearchHistory: boolean
    enableSearchSuggestions: boolean
    enableSearchAnalytics: boolean
    defaultSearchMode: 'keyword' | 'semantic' | 'hybrid'
    resultsPerPage: number
    enableSearchFilters: boolean
    enableSearchSort: boolean
    enableSearchExport: boolean
    enableSearchSharing: boolean
    enableSearchBookmarks: boolean
    enableSearchNotifications: boolean
    enableSearchPrivacy: boolean
    enableSearchSecurity: boolean
  }
}

interface SearchActions {
  // Search actions
  setQuery: (query: string) => void
  setResults: (results: SearchResult[]) => void
  setLoading: (loading: boolean) => void
  setTotalCount: (count: number) => void
  setError: (error: string | null) => void
  
  // Filter and sort actions
  setFilters: (filters: Record<string, string[]>) => void
  addFilter: (key: string, value: string) => void
  removeFilter: (key: string, value: string) => void
  clearFilters: () => void
  setSort: (sort: string) => void
  setViewMode: (mode: 'list' | 'grid') => void
  
  // Search feature actions
  setSemanticEnabled: (enabled: boolean) => void
  setAutoCompleteEnabled: (enabled: boolean) => void
  setSearchHistoryEnabled: (enabled: boolean) => void
  setSearchSuggestionsEnabled: (enabled: boolean) => void
  
  // Search history actions
  addToHistory: (query: string) => void
  removeFromHistory: (query: string) => void
  clearHistory: () => void
  setRecentSearches: (searches: string[]) => void
  setPopularSearches: (searches: string[]) => void
  
  // Search suggestions actions
  setSearchSuggestions: (suggestions: string[]) => void
  setSuggestedTags: (tags: string[]) => void
  
  // Search analytics actions
  updateAnalytics: (analytics: Partial<SearchState['searchAnalytics']>) => void
  incrementSearchCount: () => void
  addSearchTrend: (date: string, searches: number, users: number) => void
  
  // Search settings actions
  updateSettings: (settings: Partial<SearchState['searchSettings']>) => void
  resetSettings: () => void
  
  // Search operations
  search: (query: string, options?: {
    filters?: Record<string, string[]>
    sort?: string
    semantic?: boolean
  }) => Promise<void>
  loadMore: () => Promise<void>
  reset: () => void
}

type SearchStore = SearchState & SearchActions

const defaultSearchSettings: SearchState['searchSettings'] = {
  enableSemanticSearch: true,
  enableAutoComplete: true,
  enableSearchHistory: true,
  enableSearchSuggestions: true,
  enableSearchAnalytics: true,
  defaultSearchMode: 'hybrid',
  resultsPerPage: 25,
  enableSearchFilters: true,
  enableSearchSort: true,
  enableSearchExport: true,
  enableSearchSharing: true,
  enableSearchBookmarks: true,
  enableSearchNotifications: false,
  enableSearchPrivacy: true,
  enableSearchSecurity: true
}

const defaultSearchAnalytics: SearchState['searchAnalytics'] = {
  totalSearches: 0,
  uniqueUsers: 0,
  averageResponseTime: 0,
  topQueries: [],
  searchTrends: []
}

export const useMobileSearchStore = create<SearchStore>()(
  persist(
    (set, get) => ({
      // Initial state
      query: '',
      results: [],
      loading: false,
      totalCount: 0,
      error: null,
      
      filters: {},
      sort: 'relevance',
      viewMode: 'list',
      
      semanticEnabled: true,
      autoCompleteEnabled: true,
      searchHistoryEnabled: true,
      searchSuggestionsEnabled: true,
      
      searchHistory: [],
      recentSearches: [],
      popularSearches: [],
      
      searchSuggestions: [],
      suggestedTags: [],
      
      searchAnalytics: defaultSearchAnalytics,
      searchSettings: defaultSearchSettings,
      
      // Actions
      setQuery: (query) => set({ query }),
      setResults: (results) => set({ results }),
      setLoading: (loading) => set({ loading }),
      setTotalCount: (totalCount) => set({ totalCount }),
      setError: (error) => set({ error }),
      
      setFilters: (filters) => set({ filters }),
      addFilter: (key, value) => set((state) => ({
        filters: {
          ...state.filters,
          [key]: [...(state.filters[key] || []), value]
        }
      })),
      removeFilter: (key, value) => set((state) => ({
        filters: {
          ...state.filters,
          [key]: (state.filters[key] || []).filter(v => v !== value)
        }
      })),
      clearFilters: () => set({ filters: {} }),
      setSort: (sort) => set({ sort }),
      setViewMode: (viewMode) => set({ viewMode }),
      
      setSemanticEnabled: (semanticEnabled) => set({ semanticEnabled }),
      setAutoCompleteEnabled: (autoCompleteEnabled) => set({ autoCompleteEnabled }),
      setSearchHistoryEnabled: (searchHistoryEnabled) => set({ searchHistoryEnabled }),
      setSearchSuggestionsEnabled: (searchSuggestionsEnabled) => set({ searchSuggestionsEnabled }),
      
      addToHistory: (query) => set((state) => {
        if (!query.trim() || !state.searchHistoryEnabled) return state
        
        const filtered = state.searchHistory.filter(q => q !== query)
        const newHistory = [query, ...filtered].slice(0, 50)
        
        const filteredRecent = state.recentSearches.filter(q => q !== query)
        const newRecent = [query, ...filteredRecent].slice(0, 10)
        
        return {
          searchHistory: newHistory,
          recentSearches: newRecent
        }
      }),
      removeFromHistory: (query) => set((state) => ({
        searchHistory: state.searchHistory.filter(q => q !== query),
        recentSearches: state.recentSearches.filter(q => q !== query)
      })),
      clearHistory: () => set({ searchHistory: [], recentSearches: [] }),
      setRecentSearches: (recentSearches) => set({ recentSearches }),
      setPopularSearches: (popularSearches) => set({ popularSearches }),
      
      setSearchSuggestions: (searchSuggestions) => set({ searchSuggestions }),
      setSuggestedTags: (suggestedTags) => set({ suggestedTags }),
      
      updateAnalytics: (analytics) => set((state) => ({
        searchAnalytics: { ...state.searchAnalytics, ...analytics }
      })),
      incrementSearchCount: () => set((state) => ({
        searchAnalytics: {
          ...state.searchAnalytics,
          totalSearches: state.searchAnalytics.totalSearches + 1
        }
      })),
      addSearchTrend: (date, searches, users) => set((state) => ({
        searchAnalytics: {
          ...state.searchAnalytics,
          searchTrends: [
            ...state.searchAnalytics.searchTrends,
            { date, searches, users }
          ].slice(-30) // Keep last 30 days
        }
      })),
      
      updateSettings: (settings) => set((state) => ({
        searchSettings: { ...state.searchSettings, ...settings }
      })),
      resetSettings: () => set({ searchSettings: defaultSearchSettings }),
      
      search: async (query, options = {}) => {
        const state = get()
        
        if (!query.trim()) return
        
        set({ loading: true, error: null })
        
        try {
          // Update query and options
          set({
            query,
            filters: options.filters || state.filters,
            sort: options.sort || state.sort,
            semanticEnabled: options.semantic !== undefined ? options.semantic : state.semanticEnabled
          })
          
          // Add to history
          get().addToHistory(query)
          
          // Update analytics
          get().incrementSearchCount()
          get().addSearchTrend(
            new Date().toISOString().split('T')[0],
            1,
            1
          )
          
          // Implement actual search API call
          const response = await mobileSearchApi.search({
            query: query.trim(),
            filters: get().filters,
            sort: get().sort,
            limit: 20,
            offset: 0
          })
          
          set({
            results: response.results,
            totalCount: response.total,
            loading: false
          })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Search failed',
            loading: false
          })
        }
      },
      
      loadMore: async () => {
        const state = get()
        
        if (state.loading || state.results.length >= state.totalCount) return
        
        set({ loading: true })
        
        try {
          // Implement pagination
          const currentOffset = state.results.length
          const response = await mobileSearchApi.search({
            query: state.query,
            filters: state.filters,
            sort: state.sort,
            limit: 20,
            offset: currentOffset
          })
          
          set({
            results: [...state.results, ...response.results],
            loading: false
          })
        } catch (error) {
          set({
            error: error instanceof Error ? error.message : 'Load more failed',
            loading: false
          })
        }
      },
      
      reset: () => set({
        query: '',
        results: [],
        loading: false,
        totalCount: 0,
        error: null,
        filters: {},
        sort: 'relevance',
        viewMode: 'list',
        semanticEnabled: true
      })
    }),
    {
      name: 'mobile-search-store',
      partialize: (state) => ({
        searchHistory: state.searchHistory,
        recentSearches: state.recentSearches,
        popularSearches: state.popularSearches,
        searchAnalytics: state.searchAnalytics,
        searchSettings: state.searchSettings
      })
    }
  )
)

export default useMobileSearchStore
