// Mobile-optimized semantic search page
// Week 8: Mobile/responsive UX with PWA support

import React, { useState, useEffect, useCallback } from 'react'
import { 
  Search, 
  Brain, 
  Lightbulb, 
  Tag, 
  TrendingUp,
  Filter,
  SortAsc,
  Grid,
  List,
  ChevronDown
} from 'lucide-react'

interface SemanticSearchResult {
  id: string
  name: string
  path: string
  description: string
  similarity: number
  tags: string[]
  suggestedTags: string[]
  type: 'file' | 'directory'
  lastModified: string
  author: string
  size: number
}

interface SuggestedTag {
  tag: string
  confidence: number
  source: 'ner' | 'ml' | 'user'
}

const MobileSemanticSearchPage: React.FC = () => {
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SemanticSearchResult[]>([])
  const [suggestedTags, setSuggestedTags] = useState<SuggestedTag[]>([])
  const [loading, setLoading] = useState(false)
  const [showFilters, setShowFilters] = useState(false)
  const [showSort, setShowSort] = useState(false)
  const [viewMode, setViewMode] = useState<'list' | 'grid'>('list')
  const [semanticEnabled, setSemanticEnabled] = useState(true)

  // Handle search
  const handleSearch = useCallback(async () => {
    if (!query.trim()) return

    setLoading(true)
    try {
      // TODO: Replace with actual API call
      const mockResults: SemanticSearchResult[] = [
        {
          id: '1',
          name: 'customer_analytics.csv',
          path: '/datasets/customer_analytics.csv',
          description: 'Customer behavior analysis and segmentation data',
          similarity: 0.95,
          tags: ['analytics', 'customers', 'behavior'],
          suggestedTags: ['segmentation', 'demographics', 'purchase_patterns'],
          type: 'file',
          lastModified: new Date().toISOString(),
          author: 'John Doe',
          size: 1024000
        },
        {
          id: '2',
          name: 'sales_performance.json',
          path: '/reports/sales_performance.json',
          description: 'Monthly sales performance metrics and KPIs',
          similarity: 0.87,
          tags: ['sales', 'performance', 'metrics'],
          suggestedTags: ['kpi', 'revenue', 'growth'],
          type: 'file',
          lastModified: new Date().toISOString(),
          author: 'Jane Smith',
          size: 512000
        },
        {
          id: '3',
          name: 'marketing_campaigns/',
          path: '/campaigns/marketing_campaigns/',
          description: 'Marketing campaign data and performance analysis',
          similarity: 0.82,
          tags: ['marketing', 'campaigns', 'performance'],
          suggestedTags: ['roi', 'conversion', 'engagement'],
          type: 'directory',
          lastModified: new Date().toISOString(),
          author: 'Bob Johnson',
          size: 0
        }
      ]

      const mockSuggestedTags: SuggestedTag[] = [
        { tag: 'analytics', confidence: 0.95, source: 'ml' },
        { tag: 'customers', confidence: 0.89, source: 'ner' },
        { tag: 'behavior', confidence: 0.82, source: 'ml' },
        { tag: 'segmentation', confidence: 0.78, source: 'ml' },
        { tag: 'demographics', confidence: 0.75, source: 'ner' }
      ]

      setResults(mockResults)
      setSuggestedTags(mockSuggestedTags)
    } catch (error) {
      console.error('Failed to perform semantic search:', error)
    } finally {
      setLoading(false)
    }
  }, [query])

  // Handle tag suggestion click
  const handleTagClick = useCallback((tag: string) => {
    setQuery(prev => prev + (prev ? ' ' : '') + tag)
  }, [])

  // Handle result click
  const handleResultClick = useCallback((result: SemanticSearchResult) => {
    if (result.type === 'directory') {
      // Navigate to directory
      window.location.href = `/repositories${result.path}`
    } else {
      // Open file or show details
      window.location.href = `/repositories${result.path}`
    }
  }, [])

  // Format similarity score
  const formatSimilarity = (score: number) => {
    return Math.round(score * 100) + '%'
  }

  // Format file size
  const formatFileSize = (bytes: number) => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
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

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <div className="bg-white border-b border-gray-200 px-4 py-3">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-lg font-semibold text-gray-900">
              Semantic Search
            </h1>
            <p className="text-sm text-gray-600">
              AI-powered content discovery
            </p>
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setSemanticEnabled(!semanticEnabled)}
              className={`flex items-center space-x-2 px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                semanticEnabled
                  ? 'bg-blue-100 text-blue-700 border border-blue-200'
                  : 'bg-gray-100 text-gray-700 border border-gray-200'
              }`}
            >
              <Brain className="w-4 h-4" />
              <span>AI</span>
            </button>
          </div>
        </div>
      </div>

      {/* Search Bar */}
      <div className="bg-white border-b border-gray-200 px-4 py-3">
        <div className="relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleSearch()}
            placeholder="Describe what you're looking for..."
            className="w-full pl-10 pr-4 py-3 border border-gray-200 rounded-lg text-base focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <button
            onClick={handleSearch}
            disabled={!query.trim() || loading}
            className="absolute right-2 top-1/2 transform -translate-y-1/2 p-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
          >
            <Search className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Suggested Tags */}
      {suggestedTags.length > 0 && (
        <div className="bg-white border-b border-gray-200 px-4 py-3">
          <div className="flex items-center space-x-2 mb-2">
            <Lightbulb className="w-4 h-4 text-yellow-500" />
            <span className="text-sm font-medium text-gray-700">Suggested Tags</span>
          </div>
          <div className="flex flex-wrap gap-2">
            {suggestedTags.map((suggestion, index) => (
              <button
                key={index}
                onClick={() => handleTagClick(suggestion.tag)}
                className="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800 hover:bg-blue-200 transition-colors"
              >
                <Tag className="w-3 h-3 mr-1" />
                {suggestion.tag}
                <span className="ml-1 text-blue-600">
                  {Math.round(suggestion.confidence * 100)}%
                </span>
              </button>
            ))}
          </div>
        </div>
      )}

      {/* Filters and Sort */}
      <div className="bg-white border-b border-gray-200 px-4 py-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <button
              onClick={() => setShowFilters(!showFilters)}
              className={`flex items-center px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                showFilters
                  ? 'bg-blue-100 text-blue-700 border border-blue-200'
                  : 'bg-gray-100 text-gray-700 border border-gray-200 hover:bg-gray-200'
              }`}
            >
              <Filter className="w-4 h-4 mr-1" />
              Filters
            </button>
            
            <button
              onClick={() => setShowSort(!showSort)}
              className={`flex items-center px-3 py-2 rounded-md text-sm font-medium transition-colors ${
                showSort
                  ? 'bg-blue-100 text-blue-700 border border-blue-200'
                  : 'bg-gray-100 text-gray-700 border border-gray-200 hover:bg-gray-200'
              }`}
            >
              <SortAsc className="w-4 h-4 mr-1" />
              Sort
            </button>
          </div>
          
          <div className="flex items-center space-x-2">
            <span className="text-sm text-gray-600">
              {results.length} results
            </span>
            <div className="flex items-center bg-gray-100 rounded-md p-1">
              <button
                onClick={() => setViewMode('list')}
                className={`p-1 rounded ${viewMode === 'list' ? 'bg-white shadow-sm' : ''}`}
              >
                <List className="w-4 h-4" />
              </button>
              <button
                onClick={() => setViewMode('grid')}
                className={`p-1 rounded ${viewMode === 'grid' ? 'bg-white shadow-sm' : ''}`}
              >
                <Grid className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Results */}
      <div className="p-4">
        {loading ? (
          <div className="space-y-4">
            {[...Array(3)].map((_, index) => (
              <div key={index} className="animate-pulse">
                <div className="bg-white rounded-lg border border-gray-200 p-4">
                  <div className="flex items-start space-x-3">
                    <div className="w-10 h-10 bg-gray-200 rounded"></div>
                    <div className="flex-1 space-y-2">
                      <div className="h-4 bg-gray-200 rounded w-3/4"></div>
                      <div className="h-3 bg-gray-200 rounded w-1/2"></div>
                      <div className="h-3 bg-gray-200 rounded w-1/4"></div>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : results.length === 0 ? (
          <div className="text-center text-gray-500 py-8">
            <Brain className="w-12 h-12 mx-auto mb-3 text-gray-300" />
            <p className="text-sm">No results found</p>
            <p className="text-xs mt-2">Try describing what you're looking for in natural language</p>
          </div>
        ) : (
          <div className={viewMode === 'grid' ? 'grid grid-cols-2 gap-4' : 'space-y-4'}>
            {results.map((result) => (
              <div
                key={result.id}
                className={`bg-white rounded-lg border border-gray-200 ${
                  viewMode === 'list' ? 'p-4' : 'p-3'
                } hover:shadow-md transition-shadow`}
                onClick={() => handleResultClick(result)}
              >
                <div className="flex items-start space-x-3">
                  {/* Similarity Score */}
                  <div className="flex-shrink-0">
                    <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                      <span className="text-xs font-bold text-blue-700">
                        {formatSimilarity(result.similarity)}
                      </span>
                    </div>
                  </div>

                  {/* Content */}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-start justify-between">
                      <div className="flex-1 min-w-0">
                        <h3 className="text-sm font-medium text-gray-900 truncate">
                          {result.name}
                        </h3>
                        <p className="text-xs text-gray-600 mt-1 line-clamp-2">
                          {result.description}
                        </p>
                      </div>
                    </div>
                    
                    {/* Tags */}
                    <div className="mt-2 flex flex-wrap gap-1">
                      {result.tags.slice(0, 3).map((tag, index) => (
                        <span
                          key={index}
                          className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-gray-100 text-gray-700"
                        >
                          <Tag className="w-3 h-3 mr-1" />
                          {tag}
                        </span>
                      ))}
                      {result.tags.length > 3 && (
                        <span className="text-xs text-gray-500">
                          +{result.tags.length - 3} more
                        </span>
                      )}
                    </div>

                    {/* Suggested Tags */}
                    {result.suggestedTags.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-1">
                        {result.suggestedTags.slice(0, 2).map((tag, index) => (
                          <span
                            key={index}
                            className="inline-flex items-center px-2 py-1 rounded-full text-xs bg-blue-100 text-blue-800"
                          >
                            <Lightbulb className="w-3 h-3 mr-1" />
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}

                    {/* Metadata */}
                    <div className="mt-2 flex items-center space-x-4 text-xs text-gray-500">
                      {result.size > 0 && (
                        <span>{formatFileSize(result.size)}</span>
                      )}
                      <span>{formatDate(result.lastModified)}</span>
                      <span>{result.author}</span>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

export default MobileSemanticSearchPage
