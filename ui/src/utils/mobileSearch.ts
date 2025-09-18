// Mobile-optimized search utilities
// Week 8: Mobile/responsive UX with PWA support

import { SearchResult, SearchQuery, SearchResponse } from '@/types/mobileSearch'

// Format file size
export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
}

// Format date
export const formatDate = (date: string): string => {
  const d = new Date(date)
  const now = new Date()
  const diffTime = Math.abs(now.getTime() - d.getTime())
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24))
  
  if (diffDays === 1) return 'Yesterday'
  if (diffDays < 7) return `${diffDays} days ago`
  if (diffDays < 30) return `${Math.ceil(diffDays / 7)} weeks ago`
  return d.toLocaleDateString()
}

// Format similarity score
export const formatSimilarity = (score: number): string => {
  return Math.round(score * 100) + '%'
}

// Debounce function
export const debounce = <T extends (...args: any[]) => any>(
  func: T,
  wait: number
): ((...args: Parameters<T>) => void) => {
  let timeout: NodeJS.Timeout | null = null
  
  return (...args: Parameters<T>) => {
    if (timeout) clearTimeout(timeout)
    timeout = setTimeout(() => func(...args), wait)
  }
}

// Throttle function
export const throttle = <T extends (...args: any[]) => any>(
  func: T,
  limit: number
): ((...args: Parameters<T>) => void) => {
  let inThrottle: boolean = false
  
  return (...args: Parameters<T>) => {
    if (!inThrottle) {
      func(...args)
      inThrottle = true
      setTimeout(() => inThrottle = false, limit)
    }
  }
}

// Search query validation
export const validateSearchQuery = (query: string): boolean => {
  return query.trim().length > 0 && query.trim().length <= 1000
}

// Filter validation
export const validateFilters = (filters: Record<string, string[]>): boolean => {
  return Object.values(filters).every(values => 
    Array.isArray(values) && values.every(value => typeof value === 'string')
  )
}

// Sort validation
export const validateSort = (sort: string): boolean => {
  const validSorts = ['relevance', 'name_asc', 'name_desc', 'date_desc', 'size_desc']
  return validSorts.includes(sort)
}

// Search result highlighting
export const highlightSearchResult = (
  text: string,
  query: string,
  className: string = 'highlight'
): string => {
  if (!query.trim()) return text
  
  const regex = new RegExp(`(${query})`, 'gi')
  return text.replace(regex, `<span class="${className}">$1</span>`)
}

// Search result ranking
export const rankSearchResults = (
  results: SearchResult[],
  query: string
): SearchResult[] => {
  return results.sort((a, b) => {
    const aScore = calculateRelevanceScore(a, query)
    const bScore = calculateRelevanceScore(b, query)
    return bScore - aScore
  })
}

// Calculate relevance score
const calculateRelevanceScore = (result: SearchResult, query: string): number => {
  let score = 0
  const queryLower = query.toLowerCase()
  
  // Name match
  if (result.name.toLowerCase().includes(queryLower)) {
    score += 10
  }
  
  // Description match
  if (result.description?.toLowerCase().includes(queryLower)) {
    score += 5
  }
  
  // Tag match
  if (result.tags?.some(tag => tag.toLowerCase().includes(queryLower))) {
    score += 3
  }
  
  // Path match
  if (result.path.toLowerCase().includes(queryLower)) {
    score += 2
  }
  
  // Recency bonus
  const daysSinceModified = Math.floor(
    (Date.now() - new Date(result.lastModified).getTime()) / (1000 * 60 * 60 * 24)
  )
  if (daysSinceModified < 7) score += 2
  if (daysSinceModified < 30) score += 1
  
  return score
}

// Search query normalization
export const normalizeSearchQuery = (query: string): string => {
  return query
    .trim()
    .toLowerCase()
    .replace(/\s+/g, ' ')
    .replace(/[^\w\s-]/g, '')
}

// Search query expansion
export const expandSearchQuery = (query: string): string[] => {
  const expansions: string[] = [query]
  
  // Add plural/singular forms
  if (query.endsWith('s')) {
    expansions.push(query.slice(0, -1))
  } else {
    expansions.push(query + 's')
  }
  
  // Add common synonyms
  const synonyms: Record<string, string[]> = {
    'data': ['information', 'dataset', 'records'],
    'file': ['document', 'record'],
    'report': ['summary', 'analysis'],
    'customer': ['client', 'user'],
    'sales': ['revenue', 'income']
  }
  
  Object.entries(synonyms).forEach(([key, values]) => {
    if (query.includes(key)) {
      values.forEach(synonym => {
        expansions.push(query.replace(key, synonym))
      })
    }
  })
  
  return [...new Set(expansions)]
}

// Search query suggestions
export const generateSearchSuggestions = (
  query: string,
  history: string[],
  popular: string[]
): string[] => {
  const suggestions: string[] = []
  
  // Add history matches
  history.forEach(item => {
    if (item.toLowerCase().includes(query.toLowerCase())) {
      suggestions.push(item)
    }
  })
  
  // Add popular matches
  popular.forEach(item => {
    if (item.toLowerCase().includes(query.toLowerCase())) {
      suggestions.push(item)
    }
  })
  
  // Add partial matches
  const partialMatches = [...history, ...popular].filter(item =>
    item.toLowerCase().startsWith(query.toLowerCase())
  )
  
  suggestions.push(...partialMatches)
  
  return [...new Set(suggestions)].slice(0, 10)
}

// Search result grouping
export const groupSearchResults = (
  results: SearchResult[],
  groupBy: 'type' | 'repository' | 'author'
): Record<string, SearchResult[]> => {
  return results.reduce((groups, result) => {
    const key = result[groupBy] || 'Unknown'
    if (!groups[key]) {
      groups[key] = []
    }
    groups[key].push(result)
    return groups
  }, {} as Record<string, SearchResult[]>)
}

// Search result pagination
export const paginateSearchResults = (
  results: SearchResult[],
  page: number,
  pageSize: number
): {
  results: SearchResult[]
  totalPages: number
  hasNextPage: boolean
  hasPreviousPage: boolean
} => {
  const startIndex = (page - 1) * pageSize
  const endIndex = startIndex + pageSize
  const paginatedResults = results.slice(startIndex, endIndex)
  const totalPages = Math.ceil(results.length / pageSize)
  
  return {
    results: paginatedResults,
    totalPages,
    hasNextPage: page < totalPages,
    hasPreviousPage: page > 1
  }
}

// Search result export
export const exportSearchResults = (
  results: SearchResult[],
  format: 'json' | 'csv' | 'xlsx'
): string | Blob => {
  switch (format) {
    case 'json':
      return JSON.stringify(results, null, 2)
    
    case 'csv':
      const headers = ['Name', 'Path', 'Type', 'Size', 'Last Modified', 'Author', 'Tags']
      const csvRows = [
        headers.join(','),
        ...results.map(result => [
          result.name,
          result.path,
          result.type,
          result.size || 0,
          result.lastModified,
          result.author || '',
          result.tags?.join(';') || ''
        ].join(','))
      ]
      return csvRows.join('\n')
    
    case 'xlsx':
      // TODO: Implement XLSX export
      return new Blob(['XLSX export not implemented'], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet' })
    
    default:
      throw new Error(`Unsupported export format: ${format}`)
  }
}

// Search result sharing
export const generateShareUrl = (
  query: string,
  filters?: Record<string, string[]>,
  sort?: string
): string => {
  const params = new URLSearchParams()
  params.set('q', query)
  
  if (filters) {
    Object.entries(filters).forEach(([key, values]) => {
      params.set(`filter_${key}`, values.join(','))
    })
  }
  
  if (sort) {
    params.set('sort', sort)
  }
  
  return `${window.location.origin}/search?${params.toString()}`
}

// Search result bookmark
export const createBookmark = (
  name: string,
  query: string,
  filters?: Record<string, string[]>,
  sort?: string,
  description?: string,
  tags?: string[]
): {
  id: string
  name: string
  query: string
  filters?: Record<string, string[]>
  sort?: string
  description?: string
  tags?: string[]
  createdAt: string
  updatedAt: string
} => {
  return {
    id: Math.random().toString(36).substr(2, 9),
    name,
    query,
    filters,
    sort,
    description,
    tags,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString()
  }
}

// Search result notification
export const createNotification = (
  type: 'new_results' | 'query_alert' | 'system_update',
  title: string,
  message: string,
  query?: string,
  filters?: Record<string, string[]>,
  actionUrl?: string
): {
  id: string
  type: 'new_results' | 'query_alert' | 'system_update'
  title: string
  message: string
  query?: string
  filters?: Record<string, string[]>
  timestamp: string
  read: boolean
  actionUrl?: string
} => {
  return {
    id: Math.random().toString(36).substr(2, 9),
    type,
    title,
    message,
    query,
    filters,
    timestamp: new Date().toISOString(),
    read: false,
    actionUrl
  }
}

// Search result analytics
export const calculateSearchMetrics = (
  results: SearchResult[],
  query: string,
  responseTime: number
): {
  resultCount: number
  averageSize: number
  typeDistribution: Record<string, number>
  repositoryDistribution: Record<string, number>
  authorDistribution: Record<string, number>
  tagDistribution: Record<string, number>
  responseTime: number
  queryLength: number
} => {
  const resultCount = results.length
  const averageSize = results.reduce((sum, result) => sum + (result.size || 0), 0) / resultCount
  
  const typeDistribution = results.reduce((dist, result) => {
    dist[result.type] = (dist[result.type] || 0) + 1
    return dist
  }, {} as Record<string, number>)
  
  const repositoryDistribution = results.reduce((dist, result) => {
    const repo = result.repository || 'Unknown'
    dist[repo] = (dist[repo] || 0) + 1
    return dist
  }, {} as Record<string, number>)
  
  const authorDistribution = results.reduce((dist, result) => {
    const author = result.author || 'Unknown'
    dist[author] = (dist[author] || 0) + 1
    return dist
  }, {} as Record<string, number>)
  
  const tagDistribution = results.reduce((dist, result) => {
    result.tags?.forEach(tag => {
      dist[tag] = (dist[tag] || 0) + 1
    })
    return dist
  }, {} as Record<string, number>)
  
  return {
    resultCount,
    averageSize,
    typeDistribution,
    repositoryDistribution,
    authorDistribution,
    tagDistribution,
    responseTime,
    queryLength: query.length
  }
}

// Search result caching
export const createSearchCache = (
  key: string,
  value: any,
  ttl: number = 300000 // 5 minutes
): {
  key: string
  value: any
  expiresAt: string
  createdAt: string
  hits: number
  misses: number
} => {
  return {
    key,
    value,
    expiresAt: new Date(Date.now() + ttl).toISOString(),
    createdAt: new Date().toISOString(),
    hits: 0,
    misses: 0
  }
}

// Search result cache management
export const isCacheValid = (cache: any): boolean => {
  return new Date(cache.expiresAt) > new Date()
}

export const updateCacheHits = (cache: any): any => {
  return {
    ...cache,
    hits: cache.hits + 1
  }
}

export const updateCacheMisses = (cache: any): any => {
  return {
    ...cache,
    misses: cache.misses + 1
  }
}

// Search result error handling
export const createSearchError = (
  code: string,
  message: string,
  details?: any
): {
  code: string
  message: string
  details?: any
  timestamp: string
  requestId?: string
} => {
  return {
    code,
    message,
    details,
    timestamp: new Date().toISOString(),
    requestId: Math.random().toString(36).substr(2, 9)
  }
}

// Search result validation
export const validateSearchResult = (result: any): result is SearchResult => {
  return (
    typeof result === 'object' &&
    typeof result.id === 'string' &&
    typeof result.name === 'string' &&
    typeof result.path === 'string' &&
    typeof result.type === 'string' &&
    ['file', 'directory'].includes(result.type) &&
    typeof result.lastModified === 'string'
  )
}

// Search result sanitization
export const sanitizeSearchResult = (result: any): SearchResult => {
  return {
    id: String(result.id || ''),
    name: String(result.name || ''),
    path: String(result.path || ''),
    description: result.description ? String(result.description) : undefined,
    type: ['file', 'directory'].includes(result.type) ? result.type : 'file',
    size: typeof result.size === 'number' ? result.size : undefined,
    lastModified: String(result.lastModified || new Date().toISOString()),
    author: result.author ? String(result.author) : undefined,
    tags: Array.isArray(result.tags) ? result.tags.map(String) : undefined,
    repository: result.repository ? String(result.repository) : undefined,
    similarity: typeof result.similarity === 'number' ? result.similarity : undefined,
    suggestedTags: Array.isArray(result.suggestedTags) ? result.suggestedTags.map(String) : undefined
  }
}

export default {
  formatFileSize,
  formatDate,
  formatSimilarity,
  debounce,
  throttle,
  validateSearchQuery,
  validateFilters,
  validateSort,
  highlightSearchResult,
  rankSearchResults,
  normalizeSearchQuery,
  expandSearchQuery,
  generateSearchSuggestions,
  groupSearchResults,
  paginateSearchResults,
  exportSearchResults,
  generateShareUrl,
  createBookmark,
  createNotification,
  calculateSearchMetrics,
  createSearchCache,
  isCacheValid,
  updateCacheHits,
  updateCacheMisses,
  createSearchError,
  validateSearchResult,
  sanitizeSearchResult
}
