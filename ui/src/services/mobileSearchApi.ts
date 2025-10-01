// Mobile Search API Service
// Week 3: Real API integration for mobile search functionality

import { SearchResult, SearchFilters, SearchSort, SearchAnalytics, SearchSuggestion, SearchTag } from '../types/mobileSearch'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

export interface SearchRequest {
  query: string
  filters?: SearchFilters
  sort?: SearchSort
  limit?: number
  offset?: number
}

export interface SearchResponse {
  results: SearchResult[]
  total: number
  filters: SearchFilters
  suggestions: SearchSuggestion[]
  tags: SearchTag[]
}

export interface SuggestionsRequest {
  query: string
  limit?: number
}

export interface SuggestionsResponse {
  suggestions: string[]
  tags: string[]
}

export interface AnalyticsRequest {
  query: string
  filters?: SearchFilters
}

export interface AnalyticsResponse {
  totalSearches: number
  popularQueries: string[]
  recentSearches: string[]
  searchTrends: {
    date: string
    count: number
  }[]
}

export interface AlertsRequest {
  query: string
  filters?: SearchFilters
}

export interface AlertsResponse {
  alerts: {
    id: string
    type: 'new_result' | 'updated_result' | 'deleted_result'
    message: string
    timestamp: string
    data: any
  }[]
}

export interface ExportRequest {
  query: string
  filters?: SearchFilters
  format: 'csv' | 'json' | 'xlsx'
}

export interface ExportResponse {
  exportId: string
  status: 'pending' | 'processing' | 'completed' | 'failed'
  downloadUrl?: string
  expiresAt?: string
}

export interface ShareRequest {
  query: string
  filters?: SearchFilters
  recipients: string[]
  message?: string
}

export interface ShareResponse {
  shareId: string
  shareUrl: string
  expiresAt: string
}

export interface BookmarkRequest {
  query: string
  filters?: SearchFilters
  name: string
  description?: string
}

export interface BookmarkResponse {
  bookmarkId: string
  name: string
  query: string
  filters: SearchFilters
  createdAt: string
}

export interface NotificationRequest {
  limit?: number
  offset?: number
}

export interface NotificationResponse {
  notifications: {
    id: string
    type: string
    title: string
    message: string
    timestamp: string
    read: boolean
    data: any
  }[]
  total: number
}

export interface ContextRequest {
  query: string
  filters?: SearchFilters
}

export interface ContextResponse {
  context: {
    relatedQueries: string[]
    relatedTags: string[]
    relatedRepositories: string[]
    relatedAuthors: string[]
    timeRange: {
      start: string
      end: string
    }
  }
}

class MobileSearchApiService {
  private baseUrl: string
  private authToken: string | null = null

  constructor() {
    this.baseUrl = API_BASE_URL
    this.authToken = localStorage.getItem('auth_token')
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`
    
    const headers = {
      'Content-Type': 'application/json',
      ...(this.authToken && { Authorization: `Bearer ${this.authToken}` }),
      ...options.headers,
    }

    const response = await fetch(url, {
      ...options,
      headers,
    })

    if (!response.ok) {
      throw new Error(`API request failed: ${response.status} ${response.statusText}`)
    }

    return response.json()
  }

  // Search functionality
  async search(request: SearchRequest): Promise<SearchResponse> {
    const params = new URLSearchParams({
      q: request.query,
      ...(request.filters && { filters: JSON.stringify(request.filters) }),
      ...(request.sort && { sort: request.sort }),
      ...(request.limit && { limit: request.limit.toString() }),
      ...(request.offset && { offset: request.offset.toString() }),
    })

    return this.makeRequest<SearchResponse>(`/v1/search?${params}`)
  }

  // Suggestions functionality
  async getSuggestions(request: SuggestionsRequest): Promise<SuggestionsResponse> {
    const params = new URLSearchParams({
      q: request.query,
      ...(request.limit && { limit: request.limit.toString() }),
    })

    return this.makeRequest<SuggestionsResponse>(`/v1/search/suggest?${params}`)
  }

  // Analytics functionality
  async getAnalytics(request: AnalyticsRequest): Promise<AnalyticsResponse> {
    const params = new URLSearchParams({
      q: request.query,
      ...(request.filters && { filters: JSON.stringify(request.filters) }),
    })

    return this.makeRequest<AnalyticsResponse>(`/v1/search/analytics?${params}`)
  }

  // Alerts functionality
  async getAlerts(request: AlertsRequest): Promise<AlertsResponse> {
    const params = new URLSearchParams({
      q: request.query,
      ...(request.filters && { filters: JSON.stringify(request.filters) }),
    })

    return this.makeRequest<AlertsResponse>(`/v1/search/alerts?${params}`)
  }

  // Export functionality
  async exportSearch(request: ExportRequest): Promise<ExportResponse> {
    return this.makeRequest<ExportResponse>('/v1/search/export', {
      method: 'POST',
      body: JSON.stringify(request),
    })
  }

  // Sharing functionality
  async shareSearch(request: ShareRequest): Promise<ShareResponse> {
    return this.makeRequest<ShareResponse>('/v1/search/share', {
      method: 'POST',
      body: JSON.stringify(request),
    })
  }

  // Bookmarks functionality
  async createBookmark(request: BookmarkRequest): Promise<BookmarkResponse> {
    return this.makeRequest<BookmarkResponse>('/v1/search/bookmarks', {
      method: 'POST',
      body: JSON.stringify(request),
    })
  }

  async getBookmarks(): Promise<BookmarkResponse[]> {
    return this.makeRequest<BookmarkResponse[]>('/v1/search/bookmarks')
  }

  async deleteBookmark(bookmarkId: string): Promise<void> {
    return this.makeRequest<void>(`/v1/search/bookmarks/${bookmarkId}`, {
      method: 'DELETE',
    })
  }

  // Notifications functionality
  async getNotifications(request: NotificationRequest = {}): Promise<NotificationResponse> {
    const params = new URLSearchParams({
      ...(request.limit && { limit: request.limit.toString() }),
      ...(request.offset && { offset: request.offset.toString() }),
    })

    return this.makeRequest<NotificationResponse>(`/v1/notifications?${params}`)
  }

  async markNotificationAsRead(notificationId: string): Promise<void> {
    return this.makeRequest<void>(`/v1/notifications/${notificationId}/read`, {
      method: 'PUT',
    })
  }

  async markAllNotificationsAsRead(): Promise<void> {
    return this.makeRequest<void>('/v1/notifications/read-all', {
      method: 'PUT',
    })
  }

  // Context functionality
  async getContext(request: ContextRequest): Promise<ContextResponse> {
    const params = new URLSearchParams({
      q: request.query,
      ...(request.filters && { filters: JSON.stringify(request.filters) }),
    })

    return this.makeRequest<ContextResponse>(`/v1/search/context?${params}`)
  }

  // Performance functionality
  async getPerformanceMetrics(): Promise<{
    searchLatency: number
    cacheHitRate: number
    totalSearches: number
    averageResultsPerSearch: number
  }> {
    return this.makeRequest('/v1/search/performance')
  }
}

export const mobileSearchApi = new MobileSearchApiService()
